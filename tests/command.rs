macro_rules! test_filter {
    ($args: expr, $stdin: expr, $stdout: expr) => {
        test_filter!($args, $stdin, $stdout, "")
    };
    ($args: expr, $stdin: expr, $stdout: expr, $stderr: expr) => {
        let mut cmd = ::assert_cmd::Command::cargo_bin("depq")?;
        let assert = cmd.args($args).write_stdin($stdin).assert();
        assert.success().stdout($stdout).stderr($stderr);
    };
}

#[test]
fn test_show_as_text() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["show"],
        include_str!("fixtures/example.txt"),
        include_str!("fixtures/example.txt")
    );
    Ok(())
}

#[test]
fn test_show_as_text_with_file() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["show", "tests/fixtures/example.txt"],
        "",
        include_str!("fixtures/example.txt")
    );
    Ok(())
}

#[test]
fn test_show_as_text_inverted() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["show", "-I"],
        include_str!("fixtures/example.txt"),
        include_str!("fixtures/example.inverted.txt")
    );
    Ok(())
}
#[test]
fn test_show_as_json() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["show", "-t", "json"],
        include_str!("fixtures/example.txt"),
        include_str!("fixtures/example.json")
    );
    Ok(())
}

#[test]
fn test_show_as_json_with_json() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["show", "-t", "json", "tests/fixtures/example.json"],
        "",
        include_str!("fixtures/example.json")
    );
    Ok(())
}

#[test]
fn test_show_as_dot() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["show", "-t", "dot", "-R", "LR"],
        include_str!("fixtures/example.txt"),
        include_str!("fixtures/example.dot")
    );
    Ok(())
}

#[test]
fn test_dfs() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["dfs"],
        include_str!("fixtures/example.txt"),
        include_str!("fixtures/example.dfs.txt")
    );
    Ok(())
}

#[test]
fn test_dfs_with_file() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["dfs", "tests/fixtures/example.txt"],
        "",
        include_str!("fixtures/example.dfs.txt")
    );
    Ok(())
}

#[test]
fn test_dfs_tree() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["dfs", "-T"],
        include_str!("fixtures/example.txt"),
        include_str!("fixtures/example.dfs.tree.txt")
    );
    Ok(())
}

#[test]
fn test_dfs_path() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["dfs", "-P"],
        include_str!("fixtures/example.txt"),
        include_str!("fixtures/example.dfs.path.txt")
    );
    Ok(())
}

#[test]
fn test_bfs() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["bfs"],
        include_str!("fixtures/example.txt"),
        include_str!("fixtures/example.bfs.txt")
    );
    Ok(())
}

#[test]
fn test_bfs_with_file() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["bfs", "tests/fixtures/example.txt"],
        "",
        include_str!("fixtures/example.bfs.txt")
    );
    Ok(())
}

#[test]
fn test_bfs_path() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["bfs", "-P"],
        include_str!("fixtures/example.txt"),
        include_str!("fixtures/example.bfs.path.txt")
    );
    Ok(())
}

#[test]
fn test_tsort() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["tsort"],
        include_str!("fixtures/example.txt"),
        include_str!("fixtures/example.tsort.txt")
    );
    Ok(())
}

#[test]
fn test_tsort_with_file() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["tsort", "tests/fixtures/example.txt"],
        "",
        include_str!("fixtures/example.tsort.txt")
    );
    Ok(())
}

#[test]
fn test_tsort_has_loop() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = ::assert_cmd::Command::cargo_bin("depq")?;
    let assert = cmd
        .args(["tsort"])
        .write_stdin(include_str!("fixtures/has_loop.txt"))
        .assert();
    assert.failure();
    Ok(())
}
