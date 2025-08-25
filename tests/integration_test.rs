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

    let indices = [(0, 3), (1, 2), (2, 4)];
    for (word_index, new_word_index) in indices {
        let (resolved, real_word_index) = recipe
            .clone()
            .resolve(args.clone(), word_index)
            .expect("Failed to resolve recipe");
        assert_eq!(resolved, vecs!["echo", "Hello", "val1", "val0", "val2"]);
        assert_eq!(new_word_index, real_word_index);
    }
}

#[test]
fn test_resolve_2() {
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
    let args = VecDeque::from(vecs!["val0", "val1", "val2", "val3"]);

    let indices = [(0, 3), (1, 2), (2, 4), (3, 5)];
    for (word_index, new_word_index) in indices {
        let (resolved, real_word_index) = recipe
            .clone()
            .resolve(args.clone(), word_index)
            .expect("Failed to resolve recipe");
        assert_eq!(
            resolved,
            vecs!["echo", "Hello", "val1", "val0", "val2", "val3"]
        );
        assert_eq!(new_word_index, real_word_index);
    }
}
