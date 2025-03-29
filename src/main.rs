#[tokio::main]
async fn main() {
    rand_sht::quick_logger::run_qulog().await;
}

// fn errorable() {
//     let res: Result<i32, i32> = Err(0);

//     res.map(|x| x * 2)
//     .map_err(|x| x - 100)
// }

// #[cfg(test)]
// fn title_of_url_is_correct() {
//     let title =
// }
