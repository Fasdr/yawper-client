use backend::backend::BackendYawperClient;
use gui::app::EguiYawperClient;
use messages::client_message::ClientMessage;
use tokio::sync::mpsc;

mod backend;
mod gui;
mod messages;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();

    let (backend_commands_transmitter, backend_commands_receiver) =
        mpsc::channel::<ClientMessage>(100);

    let (gui_commands_transmitter, gui_commands_receiver) = mpsc::channel::<ClientMessage>(100);

    let mut yawper_backend =
        BackendYawperClient::new(backend_commands_receiver, gui_commands_transmitter);
    std::thread::spawn(move || {
        let run_time = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        run_time.block_on(async {
            yawper_backend.run().await;
        });
    });

    let yawper_gui = EguiYawperClient::new(backend_commands_transmitter, gui_commands_receiver);
    eframe::run_native(
        "Yawper",
        native_options,
        Box::new(|_cc| Ok(Box::new(yawper_gui))),
    )
}
