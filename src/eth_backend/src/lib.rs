#[cfg(test)]
mod test;

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}


struct TransactionRequest {
    to: String,
    value: u64,
}

struct TransactionResult {
    hash: String,
    status: String,
}
