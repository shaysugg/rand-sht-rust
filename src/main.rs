#[tokio::main]
async fn main() {
    rand_sht::http_client::run_http_client().await;
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
