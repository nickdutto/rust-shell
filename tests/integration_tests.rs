use rexpect::session::spawn_command;
use std::process::Command;

fn setup_env(command: &str) {
    Command::new("bash")
        .arg("-c")
        .arg(command)
        .status()
        .expect("Failed to execute setup command");
}

fn spawn_shell(path_override: Option<&str>) -> rexpect::session::PtySession {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--release", "--quiet"]);

    cmd.env("TERM", "dumb");
    cmd.env("NO_COLOR", "1");
    cmd.env("CLICOLOR", "0");

    if let Some(path) = path_override {
        let current_path = std::env::var("PATH").unwrap_or_default();
        cmd.env("PATH", format!("{}:{}", path, current_path));
    }

    spawn_command(cmd, Some(5000)).expect("Failed to spawn shell")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_xk3_pipelines_multi_command_pipelines() {
        setup_env(r#"mkdir -p "/tmp/rat""#);
        setup_env(r#"echo -e "strawberry\ngrape\norange\nmango\napple" > "/tmp/rat/file-7""#);
        setup_env(r#"mkdir -p "/tmp/fox""#);
        setup_env(r#"echo -n "apple" > "/tmp/fox/f-72""#);
        setup_env(r#"echo -n "mango" > "/tmp/fox/f-71""#);
        setup_env(r#"echo -n "pear" > "/tmp/fox/f-83""#);
        setup_env(r#"echo -n "blueberry" > "/tmp/fox/f-20""#);
        setup_env(r#"echo -n "grape" > "/tmp/fox/f-93""#);
        setup_env(r#"echo -n "pineapple" > "/tmp/fox/f-81""#);
        let mut p = spawn_shell(Some(r#"/tmp/blueberry/pineapple/strawberry"#));
        p.exp_string("$ ").unwrap();

        p.send_line("cat /tmp/rat/file-7 | head -n 3 | wc").unwrap();
        p.exp_regex(r"\s*3\s+3\s+24").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("ls /tmp/fox | tail -n 5 | head -n 3 | grep \"f-81\"")
            .unwrap();
        p.exp_string("f-81").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_ny9_pipelines_pipelines_with_built_ins() {
        let mut p = spawn_shell(Some(r#"/tmp/grape/mango/banana"#));
        p.exp_string("$ ").unwrap();

        p.send_line("echo mango-mango | wc").unwrap();
        p.exp_regex(r"\s*1\s+1\s+12").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("ls | type exit").unwrap();
        p.exp_string("exit is a shell builtin").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_br6_pipelines_dual_command_pipeline() {
        setup_env(r#"mkdir -p "/tmp/dog""#);
        setup_env(
            r#"echo -e "blueberry grape\napple pineapple\nmango strawberry\nraspberry pear\norange banana" > "/tmp/dog/file-98""#,
        );
        setup_env(r#"mkdir -p "/tmp/bee""#);
        setup_env(
            r#"echo -e "1. apple pineapple\n2. blueberry pear\n3. banana grape" > "/tmp/bee/file-70""#,
        );
        let mut p = spawn_shell(Some(r#"/tmp/blueberry/banana/mango"#));
        p.exp_string("$ ").unwrap();

        p.send_line("cat /tmp/dog/file-98 | wc").unwrap();
        p.exp_regex(r"\s*5\s+10\s+78").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("tail -f /tmp/bee/file-70 | head -n 5").unwrap();
        p.exp_string("1. apple pineapple").unwrap();
        p.exp_string("2. blueberry pear").unwrap();
        p.exp_string("3. banana grape").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_my0_parameter_expansion_expanding_empty_variables() {
        setup_env(r#"Available executables:"#);
        setup_env(r#"- custom_exe_8639"#);
        let mut p = spawn_shell(Some(r#"/tmp/fox"#));
        p.exp_string("$ ").unwrap();

        p.send_line("declare banana=orange").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line(
            "custom_exe_8639 ${missing_var_5}_suffix ${banana} ${missing_var_1} $missing_var_7",
        )
            .unwrap();
        p.exp_string("Program was passed 3 args (including program name).")
            .unwrap();
        p.exp_string("Program Signature: 6025431298").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_br2_parameter_expansion_expansion_with_braces() {
        setup_env(r#"Available executables:"#);
        setup_env(r#"- custom_exe_9478"#);
        let mut p = spawn_shell(Some(r#"/tmp/fox"#));
        p.exp_string("$ ").unwrap();

        p.send_line("declare Banana_1=orange").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("declare Mango_9=raspberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("custom_exe_9478 strawberry_${Banana_1}_pear ${Mango_9}_apple")
            .unwrap();
        p.exp_string("Program was passed 3 args (including program name).")
            .unwrap();
        p.exp_string("Program Signature: 9034386912").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_ge9_parameter_expansion_expanding_variables() {
        setup_env(r#"Available executables:"#);
        setup_env(r#"- custom_exe_2381"#);
        let mut p = spawn_shell(Some(r#"/tmp/owl"#));
        p.exp_string("$ ").unwrap();

        p.send_line("declare Blueberry_3=orange").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("declare Mango_6=raspberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("custom_exe_2381 $Blueberry_3 strawberry_$Mango_6")
            .unwrap();
        p.exp_string("Program was passed 3 args (including program name).")
            .unwrap();
        p.exp_string("Program Signature: 6749811173").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_db8_parameter_expansion_validating_variable_names() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("declare _pear=mango").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("declare 2pineapple=banana").unwrap();
        p.exp_string("declare: `2pineapple=banana': not a valid identifier")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("declare blueberry-grape=mango").unwrap();
        p.exp_string("declare: `blueberry-grape=mango': not a valid identifier")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("declare -p _pear").unwrap();
        p.exp_string("declare -- _pear=\"mango\"").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_kv5_parameter_expansion_storing_shell_variables() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("declare orange=pineapple").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("declare -p orange").unwrap();
        p.exp_string("declare -- orange=\"pineapple\"").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("declare -p missing_banana").unwrap();
        p.exp_string("declare: missing_banana: not found").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("declare orange=banana").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("declare -p orange").unwrap();
        p.exp_string("declare -- orange=\"banana\"").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_oa2_parameter_expansion_printing_missing_variables() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("declare -p missing_variable_75").unwrap();
        p.exp_string("declare: missing_variable_75: not found")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_ji0_parameter_expansion_the_declare_builtin() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("type declare").unwrap();
        p.exp_string("declare is a shell builtin").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_jv2_history_persistence_append_history_on_exit() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("echo banana grape").unwrap();
        p.exp_string("banana grape").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo apple mango").unwrap();
        p.exp_string("apple mango").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo orange grape raspberry").unwrap();
        p.exp_string("orange grape raspberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("history").unwrap();
        p.exp_string("    1  echo orange blueberry").unwrap();
        p.exp_string("    2  echo orange blueberry pear").unwrap();
        p.exp_string("    3  echo grape banana blueberry").unwrap();
        p.exp_string("    4  echo banana grape").unwrap();
        p.exp_string("    5  echo apple mango").unwrap();
        p.exp_string("    6  echo orange grape raspberry").unwrap();
        p.exp_string("    7  history").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("exit").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_kz7_history_persistence_write_history_on_exit() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("echo banana strawberry pineapple").unwrap();
        p.exp_string("banana strawberry pineapple").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo grape strawberry raspberry").unwrap();
        p.exp_string("grape strawberry raspberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("history").unwrap();
        p.exp_string("    1  echo banana strawberry pineapple")
            .unwrap();
        p.exp_string("    2  echo grape strawberry raspberry")
            .unwrap();
        p.exp_string("    3  history").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("exit").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_zp4_history_persistence_read_history_on_startup() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("history").unwrap();
        p.exp_string("    1  echo strawberry orange grape").unwrap();
        p.exp_string("    2  echo orange strawberry grape").unwrap();
        p.exp_string("    3  echo grape mango").unwrap();
        p.exp_string("    4  history").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_sx3_history_persistence_append_history_to_file() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("echo pear raspberry orange").unwrap();
        p.exp_string("pear raspberry orange").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo pineapple pear orange").unwrap();
        p.exp_string("pineapple pear orange").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo banana raspberry strawberry").unwrap();
        p.exp_string("banana raspberry strawberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("history -a /tmp/orange.txt").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo pear mango raspberry").unwrap();
        p.exp_string("pear mango raspberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("history -a /tmp/orange.txt").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_in3_history_persistence_write_history_to_file() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("echo orange banana").unwrap();
        p.exp_string("orange banana").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo orange banana").unwrap();
        p.exp_string("orange banana").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo mango raspberry").unwrap();
        p.exp_string("mango raspberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("history -w /tmp/orange.txt").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_za2_history_persistence_read_history_from_file() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("history -r /tmp/mango.txt").unwrap();
        p.exp_string("    1  history -r /tmp/mango.txt").unwrap();
        p.exp_string("    2  echo mango pear pineapple").unwrap();
        p.exp_string("    3  echo raspberry pear").unwrap();
        p.exp_string("    4  history").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_dm2_history_executing_commands_from_history() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("echo blueberry raspberry").unwrap();
        p.exp_string("blueberry raspberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo pineapple strawberry").unwrap();
        p.exp_string("pineapple strawberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("invalid_mango_command").unwrap();
        p.exp_string("invalid_mango_command: command not found")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo strawberry banana").unwrap();
        p.exp_string("strawberry banana").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo strawberry banana").unwrap();
        p.exp_string("strawberry banana").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_vq0_history_down_arrow_navigation() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("echo apple raspberry").unwrap();
        p.exp_string("apple raspberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo strawberry orange").unwrap();
        p.exp_string("strawberry orange").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("invalid_apple_command").unwrap();
        p.exp_string("invalid_apple_command: command not found")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo banana grape").unwrap();
        p.exp_string("banana grape").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo banana grape").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_rh7_history_up_arrow_navigation() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("echo apple pineapple").unwrap();
        p.exp_string("apple pineapple").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo grape pineapple").unwrap();
        p.exp_string("grape pineapple").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("invalid_orange_command").unwrap();
        p.exp_string("invalid_orange_command: command not found")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo grape blueberry").unwrap();
        p.exp_string("grape blueberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo grape pineapple").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_ag6_history_limiting_history_entries() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("echo raspberry pear").unwrap();
        p.exp_string("raspberry pear").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo apple strawberry").unwrap();
        p.exp_string("apple strawberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo mango pear").unwrap();
        p.exp_string("mango pear").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("history 2").unwrap();
        p.exp_string("    3  echo mango pear").unwrap();
        p.exp_string("    4  history 2").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo raspberry grape").unwrap();
        p.exp_string("raspberry grape").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo pear raspberry").unwrap();
        p.exp_string("pear raspberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo blueberry banana").unwrap();
        p.exp_string("blueberry banana").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo blueberry orange").unwrap();
        p.exp_string("blueberry orange").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo pear blueberry").unwrap();
        p.exp_string("pear blueberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo blueberry orange").unwrap();
        p.exp_string("blueberry orange").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("history 4").unwrap();
        p.exp_string("    8  echo blueberry orange").unwrap();
        p.exp_string("    9  echo pear blueberry").unwrap();
        p.exp_string("    10  echo blueberry orange").unwrap();
        p.exp_string("    11  history 4").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_yf5_history_listing_history() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("echo apple blueberry").unwrap();
        p.exp_string("apple blueberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo blueberry strawberry").unwrap();
        p.exp_string("blueberry strawberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo grape blueberry").unwrap();
        p.exp_string("grape blueberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("history").unwrap();
        p.exp_string("    1  echo apple blueberry").unwrap();
        p.exp_string("    2  echo blueberry strawberry").unwrap();
        p.exp_string("    3  echo grape blueberry").unwrap();
        p.exp_string("    4  history").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_bq4_history_the_history_builtin() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("type history").unwrap();
        p.exp_string("history is a shell builtin").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_dk5_background_jobs_list_multiple_jobs() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("sleep 100 &").unwrap();
        p.exp_regex(r"\[\d+\] \d+").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("jobs").unwrap();
        p.exp_string("[1]+  Running                  sleep 100 &")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("sleep 200 &").unwrap();
        p.exp_regex(r"\[\d+\] \d+").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("jobs").unwrap();
        p.exp_string("[1]-  Running                  sleep 100 &")
            .unwrap();
        p.exp_string("[2]+  Running                  sleep 200 &")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("sleep 300 &").unwrap();
        p.exp_regex(r"\[\d+\] \d+").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("jobs").unwrap();
        p.exp_string("[1]   Running                  sleep 100 &")
            .unwrap();
        p.exp_string("[2]-  Running                  sleep 200 &")
            .unwrap();
        p.exp_string("[3]+  Running                  sleep 300 &")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_jd6_background_jobs_list_a_single_job() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("sleep 100 &").unwrap();
        p.exp_regex(r"\[\d+\] \d+").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("jobs").unwrap();
        p.exp_string("[1]+  Running                  sleep 100 &")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_at7_background_jobs_starting_background_jobs() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("sleep 500 &").unwrap();
        p.exp_regex(r"\[\d+\] \d+").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_af3_background_jobs_the_jobs_builtin() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("type jobs").unwrap();
        p.exp_string("jobs is a shell builtin").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("jobs").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_tz2_programmable_completion_unregister_a_completion() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("complete  -C  /tmp/cow/singleCompleter  git")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("complete -r git").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("complete -p git").unwrap();
        p.exp_string("complete: git: no completion specification")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("git ").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_xz3_programmable_completion_longest_common_prefix() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("complete -C /tmp/dog/lcpEnvCompleter git")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("git checkout ").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_ep2_programmable_completion_multiple_completer_candidates() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("complete -C /tmp/fox/multiCandidateEnvCompleter git")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("git sta").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("git sta").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_nr7_programmable_completion_passing_environment_variables() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("complete -C /tmp/dog/gitStashCompleter git")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("git stash list ").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_zi0_programmable_completion_passing_command_line_arguments() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("complete -C /tmp/fox/gitRemoteCompleter git")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("git remote add ").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_qf1_programmable_completion_handling_no_completions() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("complete -C /tmp/dog/noCandidatesCompleter systemctl")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("systemctl xyz").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_pm5_programmable_completion_single_completion() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("complete -C /tmp/owl/singleCompleter docker")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("docker exec ").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_wl6_programmable_completion_displaying_registered_specifications() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("complete  -C  /tmp/pear.py  git").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("complete  -C  /tmp/raspberry.py  docker")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("complete -p git").unwrap();
        p.exp_string("complete -C '/tmp/pear.py' git").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("complete -p docker").unwrap();
        p.exp_string("complete -C '/tmp/raspberry.py' docker")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_oi7_programmable_completion_printing_missing_specifications() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("complete -p git").unwrap();
        p.exp_string("complete: git: no completion specification")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_ne7_programmable_completion_register_complete_builtin() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("type complete").unwrap();
        p.exp_string("complete is a shell builtin").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_bf8_filename_completion_multi_argument_completions() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("du cow_").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("du cow_4.txt cow_8/ missing_entry-458")
            .unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_jp8_filename_completion_partial_completions() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("du xyz_owl_rat_bee.txt ").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_no5_filename_completion_multiple_matches() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("stat bee_").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("stat bee_3/").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_vs5_filename_completion_missing_completions() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("stat missing_636").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_lc6_filename_completion_directory_completion() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("stat rat/owl/").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_ue6_filename_completion_nested_file_completion() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("stat banana/raspberry/apple.txt ").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_zv2_filename_completion_file_completion() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("wc blueberry-79.txt ").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_wt6_command_completion_partial_completions() {
        setup_env(r#"Available executables:"#);
        setup_env(r#"- xyz_fox"#);
        setup_env(r#"- xyz_fox_ant"#);
        setup_env(r#"- xyz_fox_ant_dog"#);
        let mut p = spawn_shell(Some(r#"/tmp/pig"#));
        p.exp_string("$ ").unwrap();

        p.send_line("xyz_fox_ant_dog ").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_wh6_command_completion_multiple_completions() {
        setup_env(r#"Available executables:"#);
        setup_env(r#"- xyz_dog"#);
        setup_env(r#"- xyz_owl"#);
        setup_env(r#"- xyz_cow"#);
        let mut p = spawn_shell(Some(r#"/tmp/owl"#));
        p.exp_string("$ ").unwrap();

        p.send_line("xyz_").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("xyz_").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_gy5_command_completion_executable_completion() {
        setup_env(r#"Available executables:"#);
        setup_env(r#"- custom_exe_9602"#);
        let mut p = spawn_shell(Some(r#"/tmp/pig"#));
        p.exp_string("$ ").unwrap();

        p.send_line("custom_exe_9602 ").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_qm8_command_completion_missing_completions() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("xyz").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_gm9_command_completion_completion_with_arguments() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("echo hello").unwrap();
        p.exp_string("hello").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo foo bar").unwrap();
        p.exp_string("foo bar").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_qp2_command_completion_builtin_completion() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("echo ").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("exit ").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_un3_redirection_append_stderr() {
        let mut p = spawn_shell(Some(r#"/tmp/mango/grape/apple"#));
        p.exp_string("$ ").unwrap();

        p.send_line("ls -1 nonexistent >> /tmp/rat/owl.md").unwrap();
        p.exp_string("ls: nonexistent: No such file or directory")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("ls -1 nonexistent 2>> /tmp/rat/pig.md")
            .unwrap();
        p.exp_string("ls: nonexistent: No such file or directory")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo Maria says Error 2>> /tmp/rat/rat.md")
            .unwrap();
        p.exp_string("Maria says Error").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("cat nonexistent 2>> /tmp/rat/rat.md").unwrap();
        p.exp_string("cat: nonexistent: No such file or directory")
            .unwrap();
        p.exp_string("ls: nonexistent: No such file or directory")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_el9_redirection_append_stdout() {
        setup_env(r#"mkdir -p "/tmp/rat""#);
        setup_env(r#"echo "orange" > "/tmp/rat/orange""#);
        setup_env(r#"echo "pear" > "/tmp/rat/pear""#);
        setup_env(r#"echo "strawberry" > "/tmp/rat/strawberry""#);
        let mut p = spawn_shell(Some(r#"/tmp/strawberry/blueberry/mango"#));
        p.exp_string("$ ").unwrap();

        p.send_line("ls -1 /tmp/rat >> /tmp/fox/fox.md").unwrap();
        p.exp_string("orange").unwrap();
        p.exp_string("pear").unwrap();
        p.exp_string("strawberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo Hello David 1>> /tmp/fox/owl.md").unwrap();
        p.exp_string("Hello David").unwrap();
        p.exp_string("Hello Alice").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo List of files: > /tmp/fox/rat.md")
            .unwrap();
        p.exp_string("List of files:").unwrap();
        p.exp_string("orange").unwrap();
        p.exp_string("pear").unwrap();
        p.exp_string("strawberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_vz4_redirection_redirect_stderr() {
        setup_env(r#"mkdir -p "/tmp/rat""#);
        setup_env(r#"echo "blueberry" > "/tmp/rat/blueberry""#);
        let mut p = spawn_shell(Some(r#"/tmp/strawberry/pear/blueberry"#));
        p.exp_string("$ ").unwrap();

        p.send_line("ls -1 nonexistent 2> /tmp/ant/ant.md").unwrap();
        p.exp_string("ls: nonexistent: No such file or directory")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo Emily file cannot be found 2> /tmp/ant/bee.md")
            .unwrap();
        p.exp_string("Emily file cannot be found").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("cat /tmp/rat/blueberry nonexistent 2> /tmp/ant/fox.md")
            .unwrap();
        p.exp_string("blueberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("cat /tmp/ant/fox.md").unwrap();
        p.exp_string("cat: nonexistent: No such file or directory")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_jv1_redirection_redirect_stdout() {
        setup_env(r#"mkdir -p "/tmp/owl""#);
        setup_env(r#"echo "banana" > "/tmp/owl/banana""#);
        setup_env(r#"echo "orange" > "/tmp/owl/orange""#);
        setup_env(r#"echo "strawberry" > "/tmp/owl/strawberry""#);
        let mut p = spawn_shell(Some(r#"/tmp/blueberry/raspberry/pear"#));
        p.exp_string("$ ").unwrap();

        p.send_line("ls -1 /tmp/owl > /tmp/bee/cow.md").unwrap();
        p.exp_string("banana").unwrap();
        p.exp_string("orange").unwrap();
        p.exp_string("strawberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo Hello James 1> /tmp/bee/dog.md").unwrap();
        p.exp_string("Hello James").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("cat /tmp/owl/orange nonexistent 1> /tmp/bee/owl.md")
            .unwrap();
        p.exp_string("cat: nonexistent: No such file or directory")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("cat /tmp/bee/owl.md").unwrap();
        p.exp_string("orange").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_qj0_quoting_executing_a_quoted_executable() {
        setup_env(r#"Available executables:"#);
        setup_env(r#"- 'exe  with  space'"#);
        setup_env(r#"- 'exe with "quotes"'"#);
        setup_env(r#"- "exe with 'single quotes'""#);
        setup_env(r#"- "exe with \\ backslash""#);
        setup_env(r#"mkdir -p "/tmp/owl""#);
        setup_env(r#"echo "mango raspberry." > "/tmp/owl/f1""#);
        setup_env(r#"echo "mango strawberry." > "/tmp/owl/f2""#);
        setup_env(r#"echo "pineapple banana." > "/tmp/owl/f3""#);
        setup_env(r#"echo "blueberry mango." > "/tmp/owl/f4""#);
        let mut p = spawn_shell(Some(r#"/tmp/owl"#));
        p.exp_string("$ ").unwrap();

        p.send_line("'exe  with  space' /tmp/owl/f1").unwrap();
        p.exp_string("mango raspberry.").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("'exe with \"quotes\"' /tmp/owl/f2").unwrap();
        p.exp_string("mango strawberry.").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("\"exe with 'single quotes'\" /tmp/owl/f3")
            .unwrap();
        p.exp_string("pineapple banana.").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("\"exe with \\\\ backslash\" /tmp/owl/f4")
            .unwrap();
        p.exp_string("blueberry mango.").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_gu3_quoting_backslash_within_double_quotes() {
        setup_env(r#"mkdir -p "/tmp/cow""#);
        setup_env(r#"echo -n "pineapple banana." > "/tmp/cow/number 14""#);
        setup_env(r#"echo -n "pineapple grape." > "/tmp/cow/doublequote \" 6""#);
        setup_env(r#"echo "orange pineapple." > "/tmp/cow/backslash \\ 47""#);
        let mut p = spawn_shell(Some(r#"/tmp/mango/blueberry/blueberry"#));
        p.exp_string("$ ").unwrap();

        p.send_line("echo \"example'world'\\\\'test\"").unwrap();
        p.exp_string("example'world'\\'test").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo \"example\\\"insidequotes\"world\\\"")
            .unwrap();
        p.exp_string("example\"insidequotesworld\"").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo \"mixed\\\"quote'script'\\\\\"").unwrap();
        p.exp_string("mixed\"quote'script'\\").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("cat /tmp/cow/\"number 14\" /tmp/cow/\"doublequote \\\" 6\" /tmp/cow/\"backslash \\\\ 47\"").unwrap();
        p.exp_string("pineapple banana.pineapple grape.orange pineapple.")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_le5_quoting_backslash_within_single_quotes() {
        setup_env(r#"mkdir -p "/tmp/ant""#);
        setup_env(r#"echo -n "pineapple pear." > "/tmp/ant/no slash 8""#);
        setup_env(r#"echo -n "banana strawberry." > "/tmp/ant/one slash \\31""#);
        setup_env(r#"echo "raspberry orange." > "/tmp/ant/two slashes \\36\\""#);
        let mut p = spawn_shell(Some(r#"/tmp/pear/strawberry/pear"#));
        p.exp_string("$ ").unwrap();

        p.send_line("echo 'example\\\\nshell'").unwrap();
        p.exp_string("example\\\\nshell").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo 'script\\\"hellotest\\\"example'")
            .unwrap();
        p.exp_string("script\\\"hellotest\\\"example").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo 'test\\\\nshell'").unwrap();
        p.exp_string("test\\\\nshell").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line(
            "cat /tmp/ant/'no slash 8' /tmp/ant/'one slash \\31' /tmp/ant/'two slashes \\36\\'",
        )
            .unwrap();
        p.exp_string("pineapple pear.banana strawberry.raspberry orange.")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_yt5_quoting_backslash_outside_quotes() {
        setup_env(r#"mkdir -p "/tmp/dog""#);
        setup_env(r#"echo -n "mango pineapple." > "/tmp/dog/_ignored_34""#);
        setup_env(r#"echo -n "blueberry banana." > "/tmp/dog/ignore_20""#);
        setup_env(r#"echo "mango pear." > "/tmp/dog/just_one_\\_74""#);
        let mut p = spawn_shell(Some(r#"/tmp/mango/apple/blueberry"#));
        p.exp_string("$ ").unwrap();

        p.send_line("echo shell\\ \\ \\ \\ \\ \\ world").unwrap();
        p.exp_string("shell      world").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo \\'\\\"world test\\\"\\'").unwrap();
        p.exp_string("'\"world test\"'").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo test\\nscript").unwrap();
        p.exp_string("testnscript").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("cat /tmp/dog/\\_ignored_34 /tmp/dog/ignore_\\20 /tmp/dog/just_one_\\\\_74")
            .unwrap();
        p.exp_string("mango pineapple.blueberry banana.mango pear.")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_tg6_quoting_double_quotes() {
        setup_env(r#"mkdir -p "/tmp/dog""#);
        setup_env(r#"echo -n "raspberry apple." > "/tmp/dog/f 23""#);
        setup_env(r#"echo -n "strawberry grape." > "/tmp/dog/f   29""#);
        setup_env(r#"echo "grape mango." > "/tmp/dog/f's50""#);
        let mut p = spawn_shell(Some(r#"/tmp/banana/blueberry/pear"#));
        p.exp_string("$ ").unwrap();

        p.send_line("echo \"hello script\"").unwrap();
        p.exp_string("hello script").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo \"script  shell\"  \"example\"\"hello\"")
            .unwrap();
        p.exp_string("script  shell examplehello").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo \"example\"  \"world's\"  script\"\"shell")
            .unwrap();
        p.exp_string("example world's scriptshell").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("cat \"/tmp/dog/f 23\" \"/tmp/dog/f   29\" \"/tmp/dog/f's50\"")
            .unwrap();
        p.exp_string("raspberry apple.strawberry grape.grape mango.")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_ni6_quoting_single_quotes() {
        setup_env(r#"mkdir -p "/tmp/cow""#);
        setup_env(r#"echo -n "grape orange." > "/tmp/cow/f   19""#);
        setup_env(r#"echo -n "blueberry grape." > "/tmp/cow/f   20""#);
        setup_env(r#"echo "pear pineapple." > "/tmp/cow/f   68""#);
        let mut p = spawn_shell(Some(r#"/tmp/strawberry/grape/banana"#));
        p.exp_string("$ ").unwrap();

        p.send_line("echo 'hello test'").unwrap();
        p.exp_string("hello test").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo test     example").unwrap();
        p.exp_string("test example").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo 'script     world' 'example''hello' test''shell")
            .unwrap();
        p.exp_string("script     world examplehello testshell")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("cat '/tmp/cow/f   19' '/tmp/cow/f   20' '/tmp/cow/f   68'")
            .unwrap();
        p.exp_string("grape orange.blueberry grape.pear pineapple.")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_gp4_navigation_the_cd_builtin_home_directory() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("cd /tmp/mango/pear/grape").unwrap();
        p.exp_string("/tmp/mango/pear/grape").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("cd ~").unwrap();
        p.exp_string("/tmp/blueberry/grape/grape").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_gq9_navigation_the_cd_builtin_relative_paths() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("cd /tmp/mango").unwrap();
        p.exp_string("/tmp/mango").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("cd ./mango/grape").unwrap();
        p.exp_string("/tmp/mango/mango/grape").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("cd ../../../").unwrap();
        p.exp_string("/tmp").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_ra6_navigation_the_cd_builtin_absolute_paths() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("cd /tmp/orange/apple/blueberry").unwrap();
        p.exp_string("/tmp/orange/apple/blueberry").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("cd /non-existing-directory").unwrap();
        p.exp_string("cd: /non-existing-directory: No such file or directory")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_ei0_navigation_the_pwd_builtin() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("type pwd").unwrap();
        p.exp_string("pwd is a shell builtin").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("pwd").unwrap();
        p.exp_string(std::env::current_dir().unwrap().to_str().unwrap())
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_ip1_run_a_program() {
        setup_env(r#"Available executables:"#);
        setup_env(r#"- custom_exe_7124"#);
        setup_env(r#"- custom_exe_3737"#);
        let mut p = spawn_shell(Some(r#"/tmp/dog"#));
        p.exp_string("$ ").unwrap();

        p.send_line("custom_exe_7124 James Emily").unwrap();
        p.exp_string("Program was passed 3 args (including program name).")
            .unwrap();
        p.exp_string("Program Signature: 4981261321").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("custom_exe_3737 David").unwrap();
        p.exp_string("Program was passed 2 args (including program name).")
            .unwrap();
        p.exp_string("Program Signature: 5293274687").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_mg5_locate_executable_files() {
        setup_env(r#"mkdir -p "/tmp/cow:/tmp/bee:/tmp""#);
        setup_env(r#"PATH is now: /tmp/cow:/tmp/bee:/tmp/ant:..."#);
        setup_env(r#"Files created:"#);
        setup_env(r#"mkdir -p "/tmp/ant""#);
        setup_env(r#"- /tmp/ant/my_exe (not executable)"#);
        setup_env(r#"mkdir -p "/tmp/cow""#);
        setup_env(r#"- /tmp/cow/my_exe (not executable)"#);
        setup_env(r#"mkdir -p "/tmp/bee""#);
        setup_env(r#"- /tmp/bee/my_exe (executable)"#);
        let mut p = spawn_shell(Some(r#"/tmp/cow"#));
        p.exp_string("$ ").unwrap();

        p.send_line("type cat").unwrap();
        p.exp_string("cat is /usr/bin/cat").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("type cp").unwrap();
        p.exp_string("cp is /usr/bin/cp").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("type mkdir").unwrap();
        p.exp_string("mkdir is /usr/bin/mkdir").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("type my_exe").unwrap();
        p.exp_string("my_exe is /tmp/bee/my_exe").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("type invalid_grape_command").unwrap();
        p.exp_string("invalid_grape_command: not found").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("type invalid_blueberry_command").unwrap();
        p.exp_string("invalid_blueberry_command: not found")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_ez5_implement_type() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("type echo").unwrap();
        p.exp_string("echo is a shell builtin").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("type exit").unwrap();
        p.exp_string("exit is a shell builtin").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("type type").unwrap();
        p.exp_string("type is a shell builtin").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("type invalid_apple_command").unwrap();
        p.exp_string("invalid_apple_command: not found").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("type invalid_grape_command").unwrap();
        p.exp_string("invalid_grape_command: not found").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_iz3_implement_echo() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("echo pear mango").unwrap();
        p.exp_string("pear mango").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo pineapple grape").unwrap();
        p.exp_string("pineapple grape").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("echo strawberry raspberry grape").unwrap();
        p.exp_string("strawberry raspberry grape").unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_pn5_implement_exit() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("invalid_blueberry_command").unwrap();
        p.exp_string("invalid_blueberry_command: command not found")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("exit").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_ff0_implement_a_repl() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("invalid_command_1").unwrap();
        p.exp_string("invalid_command_1: command not found")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("invalid_command_2").unwrap();
        p.exp_string("invalid_command_2: command not found")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("invalid_command_3").unwrap();
        p.exp_string("invalid_command_3: command not found")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("invalid_command_4").unwrap();
        p.exp_string("invalid_command_4: command not found")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("invalid_command_5").unwrap();
        p.exp_string("invalid_command_5: command not found")
            .unwrap();
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_cz2_handle_invalid_commands() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("invalid_blueberry_command").unwrap();
        p.exp_string("invalid_blueberry_command: command not found")
            .unwrap();
        p.exp_string("$ ").unwrap();
    }

    #[test]
    fn test_stage_oo8_print_a_prompt() {
        let mut p = spawn_shell(None);
        p.exp_string("$ ").unwrap();

        p.send_line("").unwrap();
        p.exp_string("$ ").unwrap();
    }
}
