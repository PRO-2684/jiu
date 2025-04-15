use jiu::Config;
use std::collections::VecDeque;

/// A macro to create a vector of strings from a list of literals.
macro_rules! vecs {
    ($($x:literal),*$(,)?) => {
        vec![$(String::from($x)),*]
    };
}

#[test]
fn test_resolve_1() {
    let config_str = r#"
        default = "test"
        [[recipes]]
        names = ["test", "t"]
        description = "Test recipe"
        arguments = ["arg0", "?arg1", "*arg2"]
        command = ["echo", "Hello", ["?arg1"], ["arg0"], ["*arg2"]]
    "#;
    let mut config: Config = toml::from_str(config_str).expect("Failed to parse config file");
    let recipe = config.recipes.pop().expect("Failed to get recipe");

    let args = VecDeque::from(vecs!["val0", "val1", "val2"]);
    let resolved = recipe.resolve(args).expect("Failed to resolve recipe");

    assert_eq!(resolved, vecs!["echo", "Hello", "val1", "val0", "val2"]);
}
