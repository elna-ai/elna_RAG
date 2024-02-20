use ic_cdk::{query,export_candid};

#[query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}


#[query]
fn query(name: String, q: Vec<f32>, limit: i32) -> String {
    // Placeholder implementation for demonstration
    println!("Querying for {} with parameters {:?} and limit {}", name, q, limit);

    // Dummy results, you would replace this with actual querying logic
    let results: Vec<String> = vec![
        "Result 1".to_string(),
        "Result 2".to_string(),
        "Result 3".to_string(),
    ];

    // Joining the results with newlines
    let joined_results = results.join("\n");

    // Returning the joined results
    joined_results
    
    // Returning the results
}

export_candid!();
