#[tokio::main]
async fn main() {
    rand_sht::quick_logger::run_qulog().await;

    // rand_sht::tictactoe::run_tictactoe();

    // rand_sht::http_client::run_http_client();

    // rand_sht::metadata_editor::run_metadata_editor();
}
