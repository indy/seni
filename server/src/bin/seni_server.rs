use async_std::task;
use dotenv;
use server::{start, Result};

fn main() -> Result<()> {
    dotenv::dotenv().ok();

    task::block_on(async {
        start().await?;
        Ok(())
    })
}
