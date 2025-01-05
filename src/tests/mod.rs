#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn interpret_block() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("./src/tests/cases/block.tox")
            .output()
            .unwrap();
        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "3");
        assert_eq!(lines[1], "3");
    }

    #[test]
    fn interpret_while() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("./src/tests/cases/while.tox")
            .output()
            .unwrap();
        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "1");
        assert_eq!(lines[1], "0");
    }

    #[test]
    fn interpret_while_math() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("./src/tests/cases/while_math.tox")
            .output()
            .unwrap();
        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 11);
        assert_eq!(lines[0], "10");
        assert_eq!(lines[1], "90");
        assert_eq!(lines[2], "720");
        assert_eq!(lines[3], "5040");
        assert_eq!(lines[4], "30240");
        assert_eq!(lines[5], "151200");
        assert_eq!(lines[6], "604800");
        assert_eq!(lines[7], "1814400");
        assert_eq!(lines[8], "3628800");
        assert_eq!(lines[9], "3628800");
    }

    #[test]
    fn interpret_for_loop() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("./src/tests/cases/forloop.tox")
            .output()
            .unwrap();
        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();

        let mut fibo = vec![];
        let mut a = 0;
        let mut b = 1;
        let mut temp;
        for _i in 0..21 {
            fibo.push(a);
            temp = b;
            b = a + b;
            a = temp;
        }

        assert_eq!(lines.len(), fibo.len() + 1);
        for i in 0..fibo.len() {
            assert_eq!(lines[i], fibo[i].to_string());
        }
    }

    #[test]
    fn function_defination() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("./src/tests/cases/funcdef.tox")
            .output()
            .unwrap();
        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "1");
        assert_eq!(lines[1], "2");
        assert_eq!(lines[2], "3");
    }

    #[test]
    fn function_changes_local_env() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("./src/tests/cases/func_mods_local_env.tox")
            .output()
            .unwrap();
        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "3");
    }

    #[test]
    fn function_return() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("./src/tests/cases/func_return.tox")
            .output()
            .unwrap();
        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "5");
    }

    #[test]
    fn function_return_nil() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("./src/tests/cases/func_return_nil.tox")
            .output()
            .unwrap();
        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "nil");
    }

    #[test]
    fn function_cond() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("./src/tests/cases/func_cond.tox")
            .output()
            .unwrap();
        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();
        assert_eq!(lines[0], "5");
        assert_eq!(lines[1], "1");
    }
    #[test]
    fn fibonacci_series() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("./src/tests/cases/fib.tox")
            .output()
            .unwrap();
        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();
        assert_eq!(lines[0], "1");
        assert_eq!(lines[1], "1");
        assert_eq!(lines[2], "2");
        assert_eq!(lines[3], "3");
        assert_eq!(lines[4], "5");
        assert_eq!(lines[5], "8");
        assert_eq!(lines[6], "13");
        assert_eq!(lines[7], "21");
        assert_eq!(lines[8], "34");
        assert_eq!(lines[9], "55");
        assert_eq!(lines[10], "89");
        assert_eq!(lines[11], "144");
        assert_eq!(lines[12], "233");
        assert_eq!(lines[13], "377");
        assert_eq!(lines[14], "610");
        assert_eq!(lines[15], "987");
        assert_eq!(lines[16], "1597");
        assert_eq!(lines[17], "2584");
        assert_eq!(lines[18], "4181");
        assert_eq!(lines[19], "6765");
    }
}
