use aho_corasick::AhoCorasick;
use anyhow::{Context, Result};
use inquire::{Select, Text};
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use zip::{write::FileOptions, ZipArchive, ZipWriter};

/// 地图标志位查询（保持最高效的 AhoCorasick 多模式匹配）
fn check_flags_optimized(data: &str) {
    let queries = [
        ("红水晶甲", "NAME_Crystal_Nidus"),
        ("神秘要塞", "fortress_country"),
        ("无限神机", "military_power"),
        ("好爹", "fallen_machine_empire_awaken_1"),
        ("特殊星系-杜戈尔(遗珍-楔)", "legendary_leader_spawn_system"),
        ("位面之魇", "guardians_horror_system"),
        ("幽魂", "guardians_wraith_pulsar"),
        ("噬星者", "guardians_stellarite_system"),
        ("虚空孳孽", "hatchling_egg"),
        ("蜂巢小行星", "guardians_hive_system"),
        ("时之虫", "horizonsignal_spawn"),
        ("L星团_灰蛊风暴", "gray_goo_crisis_set"),
        ("L星团_L星龙", "dragon_season"),
        ("L星团_协和国", "gray_goo_empire_set"),
    ];

    let patterns: Vec<&str> = queries.iter().map(|&(_, key)| key).collect();
    let ac = AhoCorasick::new(&patterns).unwrap();
    let mut found = vec![false; patterns.len()];

    for mat in ac.find_iter(data) {
        found[mat.pattern().as_usize()] = true;
    }

    println!("\n--- 🗺️ 地图查询结果 ---");
    for i in 0..11 {
        println!("{} = {}", queries[i].0, found[i]);
    }

    if found[11] {
        println!("L星团结果 = 灰蛊风暴 (True)");
    } else if found[12] {
        println!("L星团结果 = L星龙 (True)");
    } else if found[13] {
        println!("L星团结果 = 德萨努协和国 (True)");
    } else {
        println!("L星团结果 = 小灰 (True)");
    }
    println!("------------------------\n");
}

/// 谍报行动修改：恢复引擎级演算推演
fn modify_espionage_operations(lines: &mut Vec<String>, target_op: &str, target_id: &str) {
    let mut brace_depth = 0_i32;
    let mut in_target_block = false;
    let mut target_depth = 0_i32;
    let mut current_target_id = String::new();

    let mut in_target_op = false;
    let mut op_depth = -1_i32;
    let mut in_log = false;
    let mut log_depth = -1_i32;

    let mut current_total_info = 0;
    let mut cur_entry_info_idx: i32 = -1;
    let mut cur_entry_roll_idx: i32 = -1;
    let mut cur_skill = 0;
    let mut cur_diff = 0;

    let re_target = Regex::new(r"^\s*target\s*=").unwrap();
    let re_id = Regex::new(r"^\s*id=(\d+)").unwrap();
    let re_type = Regex::new(&format!(r#"^\s*type="{target_op}""#)).unwrap();
    let re_log = Regex::new(r"^\s*log\s*=").unwrap();
    let re_info = Regex::new(r"^\s*info=(\d+)").unwrap();
    let re_roll = Regex::new(r"^\s*roll=\d+").unwrap();
    let re_skill = Regex::new(r"skill=(\d+)").unwrap();
    let re_diff = Regex::new(r"difficulty=(\d+)").unwrap();
    let re_last_roll = Regex::new(r"^\s*last_roll=\d+").unwrap();
    let re_info_replace = Regex::new(r"info=\d+").unwrap();

    for i in 0..lines.len() {
        let open_braces = lines[i].matches('{').count() as i32;
        let close_braces = lines[i].matches('}').count() as i32;
        brace_depth += open_braces;
        let after_depth = brace_depth - close_braces;

        if re_target.is_match(&lines[i]) {
            in_target_block = true;
            target_depth = brace_depth;
        }
        if in_target_block {
            if let Some(caps) = re_id.captures(&lines[i]) {
                current_target_id = caps.get(1).unwrap().as_str().to_string();
            }
        }
        if in_target_block && lines[i].contains('}') && after_depth < target_depth {
            in_target_block = false;
        }

        if re_type.is_match(&lines[i]) {
            if current_target_id == target_id {
                in_target_op = true;
                op_depth = brace_depth - 1;
                current_total_info = 0;
            }
        }

        if in_target_op && re_log.is_match(&lines[i]) {
            in_log = true;
            log_depth = op_depth + 1;
        }

        if in_log && brace_depth > log_depth + 1 {
            if re_info.is_match(&lines[i]) {
                cur_entry_info_idx = i as i32;
            } else if re_roll.is_match(&lines[i]) {
                cur_entry_roll_idx = i as i32;
            } else if let Some(caps) = re_skill.captures(&lines[i]) {
                cur_skill = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
            } else if let Some(caps) = re_diff.captures(&lines[i]) {
                cur_diff = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
            }
        }

        if in_log && lines[i].contains('}') && after_depth == log_depth + 1 {
            if cur_entry_roll_idx != -1 && cur_entry_info_idx != -1 {
                let r_idx = cur_entry_roll_idx as usize;
                let i_idx = cur_entry_info_idx as usize;

                lines[r_idx] = re_roll.replace(&lines[r_idx], "roll=10").to_string();

                let a = 10 + cur_skill + current_total_info - cur_diff;
                let gained = if a >= 14 { 3 } else if a >= 9 { 2 } else if a >= 2 { 1 } else { 0 };
                current_total_info += gained;
                
                lines[i_idx] = re_info_replace.replace(&lines[i_idx], format!("info={}", current_total_info).as_str()).to_string();
            }

            cur_entry_info_idx = -1;
            cur_entry_roll_idx = -1;
            cur_skill = 0;
            cur_diff = 0;
        }

        if in_log && lines[i].contains('}') && after_depth == log_depth {
            in_log = false;
        }

        if in_target_op && !in_log && after_depth == op_depth + 1 {
            if re_info.is_match(&lines[i]) {
                lines[i] = re_info_replace.replace(&lines[i], format!("info={}", current_total_info).as_str()).to_string();
            } else if re_last_roll.is_match(&lines[i]) {
                let re_last_roll_replace = Regex::new(r"last_roll=\d+").unwrap();
                lines[i] = re_last_roll_replace.replace(&lines[i], "last_roll=10").to_string();
            }
        }

        if in_target_op && lines[i].contains('}') && after_depth <= op_depth {
            in_target_op = false;
            current_target_id.clear();
        }

        brace_depth = after_depth;
    }
}

/// 考古遗址修改：恢复引擎级演算推演
fn modify_archaeology_sites(lines: &mut Vec<String>, target_site: &str) {
    let mut brace_depth = 0_i32;
    let mut in_target_site = false;
    let mut site_depth = -1_i32;
    let mut in_log = false;
    let mut log_depth = -1_i32;

    let mut current_total_clues = 0;
    let mut cur_entry_clues_idx: i32 = -1;
    let mut cur_entry_roll_idx: i32 = -1;
    let mut cur_entry_total_idx: i32 = -1;
    let mut cur_bonus = 0;
    let mut cur_diff = 0;

    let re_type = Regex::new(&format!(r#"^\s*type="{target_site}""#)).unwrap();
    let re_log = Regex::new(r"^\s*log\s*=").unwrap();
    let re_clues = Regex::new(r"^\s*clues=(\d+)").unwrap();
    let re_roll = Regex::new(r"^\s*roll=\d+").unwrap();
    let re_total = Regex::new(r"^\s*total=\d+").unwrap();
    let re_bonus = Regex::new(r"^\s*bonus=(\d+)").unwrap();
    let re_diff = Regex::new(r"^\s*difficulty=(\d+)").unwrap();
    let re_last_roll = Regex::new(r"^\s*last_roll=\d+").unwrap();
    let re_clues_replace = Regex::new(r"clues=\d+").unwrap();
    let re_total_replace = Regex::new(r"total=\d+").unwrap();

    for i in 0..lines.len() {
        let open_braces = lines[i].matches('{').count() as i32;
        let close_braces = lines[i].matches('}').count() as i32;
        brace_depth += open_braces;
        let after_depth = brace_depth - close_braces;

        if re_type.is_match(&lines[i]) {
            in_target_site = true;
            site_depth = brace_depth - 1;
            current_total_clues = 0;
        }

        if in_target_site && re_log.is_match(&lines[i]) {
            in_log = true;
            log_depth = site_depth + 1;
        }

        if in_log && brace_depth > log_depth + 1 {
            if re_clues.is_match(&lines[i]) {
                cur_entry_clues_idx = i as i32;
            } else if re_roll.is_match(&lines[i]) {
                cur_entry_roll_idx = i as i32;
            } else if re_total.is_match(&lines[i]) {
                cur_entry_total_idx = i as i32;
            } else if let Some(caps) = re_bonus.captures(&lines[i]) {
                cur_bonus = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
            } else if let Some(caps) = re_diff.captures(&lines[i]) {
                cur_diff = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
            }
        }

        if in_log && lines[i].contains('}') && after_depth == log_depth + 1 {
            if cur_entry_roll_idx != -1 && cur_entry_clues_idx != -1 && cur_entry_total_idx != -1 {
                let r_idx = cur_entry_roll_idx as usize;
                let c_idx = cur_entry_clues_idx as usize;
                let t_idx = cur_entry_total_idx as usize;

                lines[r_idx] = re_roll.replace(&lines[r_idx], "roll=10").to_string();
                lines[t_idx] = re_total_replace.replace(&lines[t_idx], format!("total={}", current_total_clues).as_str()).to_string();

                let a = 10 + cur_bonus + current_total_clues - cur_diff;
                let gained = if a >= 14 { 3 } else if a >= 11 { 2 } else if a >= 6 { 1 } else { 0 };
                
                lines[c_idx] = re_clues_replace.replace(&lines[c_idx], format!("clues={}", gained).as_str()).to_string();
                current_total_clues += gained;
            }

            cur_entry_clues_idx = -1;
            cur_entry_roll_idx = -1;
            cur_entry_total_idx = -1;
            cur_bonus = 0;
            cur_diff = 0;
        }

        if in_log && lines[i].contains('}') && after_depth == log_depth {
            in_log = false;
        }

        if in_target_site && !in_log && after_depth == site_depth + 1 {
            if re_clues.is_match(&lines[i]) {
                lines[i] = re_clues_replace.replace(&lines[i], format!("clues={}", current_total_clues).as_str()).to_string();
            } else if re_last_roll.is_match(&lines[i]) {
                let re_last_roll_replace = Regex::new(r"last_roll=\d+").unwrap();
                lines[i] = re_last_roll_replace.replace(&lines[i], "last_roll=10").to_string();
            }
        }

        if in_target_site && lines[i].contains('}') && after_depth <= site_depth {
            in_target_site = false;
        }

        brace_depth = after_depth;
    }
}

/// 第一次接触修改：全新引入层级追踪，杜绝误伤
fn modify_first_contacts(lines: &mut Vec<String>, target_owner_id: &str) {
    let mut brace_depth = 0_i32;
    let mut in_first_contacts = false;
    let mut fc_depth = -1_i32;
    let mut in_target_entry = false;
    let mut entry_depth = -1_i32;

    let re_fc = Regex::new(r"^\s*first_contacts\s*=").unwrap();
    let re_owner = Regex::new(&format!(r"^\s*owner={}", target_owner_id)).unwrap();
    let re_clues = Regex::new(r"clues=\d+").unwrap();
    let re_days = Regex::new(r"days_left=\d+(?:\.\d+)?").unwrap();
    let re_last = Regex::new(r"last_roll=\d+").unwrap();

    for i in 0..lines.len() {
        let open_braces = lines[i].matches('{').count() as i32;
        let close_braces = lines[i].matches('}').count() as i32;
        brace_depth += open_braces;
        let after_depth = brace_depth - close_braces;

        if re_fc.is_match(&lines[i]) {
            in_first_contacts = true;
            fc_depth = brace_depth - 1;
        }

        // 当匹配到属于自己的 owner ID 时，锁定当前字典层级
        if in_first_contacts && re_owner.is_match(&lines[i]) {
            in_target_entry = true;
            entry_depth = brace_depth - 1;
        }

        if in_target_entry {
            if lines[i].contains("clues=") {
                lines[i] = re_clues.replace(&lines[i], "clues=20").to_string();
            } else if lines[i].contains("days_left=") {
                lines[i] = re_days.replace(&lines[i], "days_left=1").to_string();
            } else if lines[i].contains("last_roll=") {
                lines[i] = re_last.replace(&lines[i], "last_roll=10").to_string();
            }
        }

        // 离开当前接触条目时重置状态
        if in_target_entry && lines[i].contains('}') && after_depth <= entry_depth {
            in_target_entry = false;
        }

        // 离开 first_contacts 根节点时彻底重置
        if in_first_contacts && lines[i].contains('}') && after_depth <= fc_depth {
            in_first_contacts = false;
        }

        brace_depth = after_depth;
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() < 2 {
        Text::new("请输入 .sav 存档文件路径 (或直接将文件拖入此窗口):").prompt()?
    } else {
        args[1].clone()
    };
    let file_path = file_path.trim_matches(|c| c == '"' || c == '\'').to_string();

    let backup_path = format!("{}.bak", file_path);
    std::fs::copy(&file_path, &backup_path)?;
    println!("[\u{1f6e1}\u{fe0f} ] 已自动创建备份: {}", backup_path);

    let file = File::open(&file_path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut gamestate_data = String::new();
    println!("[\u{23f3}] 正在内存中解压并读取数据...");
    archive.by_name("gamestate")?.read_to_string(&mut gamestate_data)?;

    let mut meta_data = Vec::new();
    if let Ok(mut meta_file) = archive.by_name("meta") {
        meta_file.read_to_end(&mut meta_data)?;
    }
    drop(archive);

    check_flags_optimized(&gamestate_data);

    let mode = Select::new("请选择要修改的功能:", vec!["谍报行动修改", "考古遗址修改", "第一次接触修改"]).prompt()?;

    let mut target_id = String::new();
    let mut target_type = String::new();

    if mode == "谍报行动修改" {
        target_id = Text::new("请输入目标国家 ID:").with_default("16777296").prompt()?;
        let op_choice = Select::new(
            "请选择目标行动类型:",
            vec![
                "信息收集 (operation_gather_information)",
                "引爆外交事故 (operation_spark_diplomatic_incident)",
                "准备策动卧底 (operation_prepare_sleeper_cells)",
                "招募线人 (operation_acquire_asset)",
                "敲诈支持 (operation_extort_favors)",
                "诋毁运动 (operation_smear_campaign)",
                "窃取科技 (operation_steal_technology)",
                "破坏恒星基地设施 (operation_sabotage_starbase)",
                "走私人口 (operation_smuggle_pop)",
                "武装私掠 (operation_arm_privateers)",
                "手动输入",
            ],
        ).prompt()?;
        
        if op_choice == "手动输入" {
            target_type = Text::new("请手动输入谍报行动类型:").prompt()?;
        } else {
            let start = op_choice.find('(').unwrap() + 1;
            let end = op_choice.find(')').unwrap();
            target_type = op_choice[start..end].to_string();
        }
    } else if mode == "考古遗址修改" {
        let site_choice = Select::new(
            "请选择目标遗址类型:",
            vec![
                "伟大先驱 (site_grand_herald)",
                "誊录仪 (site_rubricator)",
                "沃尔陶姆 (site_vultaumar)",
                "芬·哈巴尼斯 (site_fen_habbanis)",
                "手动输入",
            ],
        ).prompt()?;
        
        if site_choice == "手动输入" {
            target_type = Text::new("请手动输入遗址类型:").prompt()?;
        } else {
            let start = site_choice.find('(').unwrap() + 1;
            let end = site_choice.find(')').unwrap();
            target_type = site_choice[start..end].to_string();
        }
    } else if mode == "第一次接触修改" {
        target_id = Text::new("请输入己方国家 ID (对应 owner ID):").with_default("0").prompt()?;
    }

    println!("[\u{2699}\u{fe0f} ] 正在分析并精确修改目标数据 (严格遵守游戏逻辑演算)...");
    
    // 转换为按行处理，恢复高精度的作用域控制
    let mut lines: Vec<String> = gamestate_data.lines().map(|s| s.to_string()).collect();
    
    if mode == "谍报行动修改" {
        modify_espionage_operations(&mut lines, &target_type, &target_id);
    } else if mode == "考古遗址修改" {
        modify_archaeology_sites(&mut lines, &target_type);
    } else if mode == "第一次接触修改" {
        modify_first_contacts(&mut lines, &target_id);
    }

    println!("[\u{23f3}] 正在重新打包并回写数据...");
    let out_file = File::create(&file_path).context("创建输出文件失败，请检查文件占用")?;
    let mut zip_writer = ZipWriter::new(out_file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    if !meta_data.is_empty() {
        zip_writer.start_file("meta", options)?;
        zip_writer.write_all(&meta_data)?;
    }

    zip_writer.start_file("gamestate", options)?;
    // 使用高效遍历直接写入，避免额外 Join 带来的内存分配峰值
    for (i, line) in lines.iter().enumerate() {
        zip_writer.write_all(line.as_bytes())?;
        if i < lines.len() - 1 {
            zip_writer.write_all(b"\n")?;
        }
    }
    zip_writer.finish()?;

    println!("[\u{2705}] 修复完成！地图查询与数据修改完毕，存档合法性已完美保护。");
    
    let _ = Text::new("按回车键退出...").prompt();
    Ok(())
}