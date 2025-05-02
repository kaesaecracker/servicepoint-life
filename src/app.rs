use crate::{
    print::{println_debug, println_info, println_warning},
    simulation::{Simulation, SimulationEvent},
    Cli,
};
use crossterm::{
    event,
    event::{Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use servicepoint::{
    Bitmap, BitmapCommand, BrightnessGrid, BrightnessGridCommand, SendCommandExt, UdpSocketExt,
    FRAME_PACING, TILE_HEIGHT, TILE_WIDTH,
};
use std::{
    io::stdout,
    net::UdpSocket,
    thread,
    time::{Duration, Instant},
};

pub(crate) struct App {
    connection: UdpSocket,
    sim: Simulation,
    target_duration: Duration,
    pixels: Bitmap,
    luma: BrightnessGrid,
    terminated: bool,
}

impl App {
    pub fn new(cli: Cli) -> Self {
        let connection = UdpSocket::bind_connect(cli.destination)
            .expect("Could not connect. Did you forget `--destination`?");

        execute!(stdout(), EnterAlternateScreen, EnableLineWrap)
            .expect("could not enter alternate screen");
        enable_raw_mode().expect("could not enable raw terminal mode");

        Self {
            connection,
            sim: Simulation::new(),
            terminated: false,
            target_duration: FRAME_PACING * 4,
            pixels: Bitmap::max_sized(),
            luma: BrightnessGrid::new(TILE_WIDTH, TILE_HEIGHT),
        }
    }

    pub(crate) fn run_iteration(&mut self) {
        let start = Instant::now();
        self.sim.run_iteration();

        self.sim.draw_state(&mut self.pixels, &mut self.luma);
        let cmd: BitmapCommand = self.pixels.clone().into();
        self.connection.send_command(cmd).unwrap();
        let cmd: BrightnessGridCommand = self.luma.clone().into();
        self.connection.send_command(cmd).unwrap();

        self.poll_events();

        let tick_time = start.elapsed();
        if tick_time < self.target_duration {
            thread::sleep(self.target_duration - tick_time);
        }
    }

    pub(crate) fn terminated(&self) -> bool {
        self.terminated
    }

    fn poll_events(&mut self) -> bool {
        while event::poll(Duration::from_secs(0)).expect("could not poll") {
            let event = event::read().expect("could not read event");

            if let Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code,
                ..
            }) = event
            {
                if let Some(sim_event) = self.handle_key(code) {
                    self.sim.handle_event(sim_event);
                };
            } else {
                println_debug(format!("unhandled event {event:?}"));
            }
        }
        false
    }

    fn handle_key(&mut self, code: KeyCode) -> Option<SimulationEvent> {
        match code {
            KeyCode::Char('d') => Some(SimulationEvent::RandomizeLeftPixels),
            KeyCode::Char('e') => Some(SimulationEvent::RandomizeLeftLuma),
            KeyCode::Char('f') => Some(SimulationEvent::RandomizeRightPixels),
            KeyCode::Char('r') => Some(SimulationEvent::RandomizeRightLuma),
            KeyCode::Right => Some(SimulationEvent::SeparatorAccelerate),
            KeyCode::Left => Some(SimulationEvent::SeparatorDecelerate),
            KeyCode::Char('h') => {
                println_info("[h] help");
                println_info("[q] quit");
                println_info("[d] randomize left pixels");
                println_info("[e] randomize left luma");
                println_info("[r] randomize right pixels");
                println_info("[f] randomize right luma");
                println_info("[→] accelerate divider right");
                println_info("[←] accelerate divider left");
                None
            }
            KeyCode::Char('q') => {
                println_warning("terminating");
                self.terminated = true;
                None
            }
            KeyCode::Up => {
                self.target_duration = self
                    .target_duration
                    .saturating_sub(Duration::from_millis(1));
                println_info(format!(
                    "increased simulation speed to {} ups",
                    1f64 / self.target_duration.as_secs_f64()
                ));
                None
            }
            KeyCode::Down => {
                self.target_duration = self
                    .target_duration
                    .saturating_add(Duration::from_millis(1));
                println_info(format!(
                    "decreased simulation speed to {} ups",
                    1f64 / self.target_duration.as_secs_f64()
                ));
                None
            }
            key_code => {
                println_debug(format!("unhandled KeyCode {key_code:?}"));
                None
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        disable_raw_mode().expect("could not disable raw terminal mode");
        execute!(stdout(), LeaveAlternateScreen).expect("could not leave alternate screen");
    }
}
