use std::io::{BufReader, Cursor, Read};
use ttx_eng::cli;

#[test]
fn process_input_success() {
    let test_cases = get_test_cases();

    for ts in test_cases {
        let reader = BufReader::new(ts.input.as_bytes());
        let mut writer = Cursor::new(Vec::new());

        cli::process_input(reader, writer.get_mut()).expect("failed to process input");

        let mut output = String::new();
        writer
            .read_to_string(&mut output)
            .expect("failed to read output");

        assert_elements_no_order(output.as_str(), ts.expected_output, ts.msg)
    }
}

fn assert_elements_no_order(a: &str, b: &str, msg: &str) {
    let mut a_vec: Vec<_> = a.lines().collect();
    let mut b_vec: Vec<_> = b.lines().collect();
    a_vec.sort();
    b_vec.sort();

    assert_eq!(a_vec, b_vec, "failed: {}", msg)
}

struct TestCase<'a> {
    input: &'a str,
    expected_output: &'a str,
    msg: &'a str,
}

fn get_test_cases<'a>() -> Vec<TestCase<'a>> {
    vec![
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0a
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0"#,
            expected_output: r#"client,available,held,total,locked
2,2.0,0,2.0,false
1,1.5,0,1.5,false
"#,
            msg: "test case invalid row",
        },
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, 1.0
dispute, 1, 1"#,
            expected_output: r#"client,available,held,total,locked
1,0.0,1.0,1.0,false
"#,
            msg: "test case dispute",
        },
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, 1.0
dispute, 1, 1,
resolve, 1, 1"#,
            expected_output: r#"client,available,held,total,locked
1,1.0,0.0,1.0,false
"#,
            msg: "test case resolution",
        },
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, 1.0
dispute, 1, 1,
chargeback, 1, 1,"#,
            expected_output: r#"client,available,held,total,locked
1,0.0,0.0,0.0,true
"#,
            msg: "test case chargeback",
        },
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, 1.0
dispute, 1, 1,
chargeback, 1, 1,
deposit, 1, 1, 1.0"#,
            expected_output: r#"client,available,held,total,locked
1,0.0,0.0,0.0,true
"#,
            msg: "test case locked account",
        },
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, 1.12345678"#,
            expected_output: r#"client,available,held,total,locked
1,1.1235,0,1.1235,false
"#,
            msg: "test case precision",
        },
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 1.0
dispute, 2, 1"#,
            expected_output: r#"client,available,held,total,locked
1,1.0,0,1.0,false
2,1.0,0,1.0,false
"#,
            msg: "test case invalid dispute client id",
        },
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 1, 1.0"#,
            expected_output: r#"client,available,held,total,locked
1,1.0,0,1.0,false
2,0,0,0,false
"#,
            msg: "test case invalid tx id",
        },
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, 1.0
withdrawal, 1, 2, 1.0
dispute, 1, 2"#,
            expected_output: r#"client,available,held,total,locked
1,0.0,0,0,false
"#,
            msg: "test case invalid dispute not a deposit",
        },
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, 1.0
resolve, 1, 1"#,
            expected_output: r#"client,available,held,total,locked
1,1.0,0,1.0,false
"#,
            msg: "test case resolution not in dispute",
        },
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, -1.0"#,
            expected_output: r#"client,available,held,total,locked
1,0,0,0,false
"#,
            msg: "test case negative deposit",
        },
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, 1.0
withdrawal, 1, 1, -1.0"#,
            expected_output: r#"client,available,held,total,locked
1,1.0,0,1.0,false
"#,
            msg: "test case negative withdrawal",
        },
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, 1.0
withdrawal, 1, 1, 4.0"#,
            expected_output: r#"client,available,held,total,locked
1,1.0,0,1.0,false
"#,
            msg: "test case withdrawal insufficient funds",
        },
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, 1.0
chargeback, 1, 1"#,
            expected_output: r#"client,available,held,total,locked
1,1.0,0,1.0,false
"#,
            msg: "test case chargeback not in dispute",
        },
        TestCase {
            input: r#"type, client, tx, amount
deposit, 1, 1, 1.0
dispute, 1, 2"#,
            expected_output: r#"client,available,held,total,locked
1,1.0,0,1.0,false
"#,
            msg: "test case dispute tx not found",
        },
    ]
}
