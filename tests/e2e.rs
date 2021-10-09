use randomlib::run;
use std::fs::{self, File, OpenOptions};

#[test]
fn compare_fixtures() {
    let paths = fs::read_dir("./tests/fixtures").unwrap();

    for path in paths {
        let fixture_dir = path.unwrap().path();
        let fixture_dir = fixture_dir.to_str().unwrap();

        let input_file_name = format!("{}/input.csv", fixture_dir);
        let run_output_file_name = format!("{}/_test_output.csv", fixture_dir);

        // The input file to read transactions from
        let input_file = File::open(input_file_name).unwrap();
        // The output file to write accounts to
        let run_output_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&run_output_file_name)
            .unwrap();

        run(input_file, run_output_file).unwrap();
        let run_output = fs::read_to_string(&run_output_file_name).unwrap();

        // Expected results
        let output_file_name = format!("{}/output.csv", fixture_dir);
        let output = fs::read_to_string(output_file_name).unwrap();

        // Compare expected with test output
        assert_eq!(
            run_output.lines().collect::<Vec<_>>().len(),
            output.lines().collect::<Vec<_>>().len()
        );

        // Get lines and skip csv header
        let expected_records = output.lines().skip(1);

        for expected_record in expected_records {
            // Check that there is a row that is equal to `expected_row`.
            // Ordering can differ from run to run, so comparing by
            // expected_record[i] = run_output[i] does not work.
            assert!(run_output
                .lines()
                .skip(1)
                .any(|output_record| { output_record == expected_record }));
        }
    }
}
