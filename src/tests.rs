#[cfg(test)]
mod tests {
    use subversion_edge_modify_tool::permissions::{Permissions, UserAuth};
    // use std::fs;
    use crate::{
        convert_address, extract_name, extract_permissions, extract_substrings_containing_base_url,
        split_string, SVNAddress,
    };
    // use prettydiff::{ diff_slice};
    // use prettydiff::basic::DiffOp;

    #[test]
    fn test_generate_permissions() {
        let mut svn = SVNAddress::new();
        let origin = r#"SVN账号名称
chenyang,wanhongchang,wanhongchang,wangrui,yangtao,makun
注明申请路径、访问权限（只读\读写\禁止访问）
申请路径、访问权限
http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/1.x/software/branches/iteration_1.0.2 读写"#;
        svn.set_old(origin);
        svn.update();
        let result = svn.generate_permissions();
        assert!(result.is_some());
        let mut expected = Vec::new();
        expected.push(Permissions{ repository: "[softwarerepo:/products/auto/inspect/tunnel_spec_vision/1.x/software/branches/iteration_1.0.2]".to_string(),
            users: vec![
                UserAuth{ user: "chenyang".to_string(), auth: "rw".to_string() },
                UserAuth{ user: "wanhongchang".to_string(), auth: "rw".to_string() },
                UserAuth{ user: "wanhongchang".to_string(), auth: "rw".to_string() },
                UserAuth{ user: "wangrui".to_string(), auth: "rw".to_string() },
                UserAuth{ user: "yangtao".to_string(), auth: "rw".to_string() },
                UserAuth{ user: "makun".to_string(), auth: "rw".to_string() },
            ] });
        assert_eq!(result.unwrap(), expected);
    }


    #[test]
    fn test_split_by_both_commas() {
        assert_eq!(
            split_string("hello,world，rust"),
            vec!["hello", "world", "rust"]
        );
        // 用 、 分割
        assert_eq!(
            split_string("hello,world，rust、c"),
            vec!["hello", "world", "rust", "c"]
        );
        // 去除首尾空格
        assert_eq!(split_string("你好，世界、hello, good"), vec!["你好", "世界","hello","good"]);
        assert_eq!(split_string("你好,世界"), vec!["你好", "世界"]);
    }

    #[test]
    fn test_no_split() {
        assert_eq!(split_string("nocommahere"), vec!["nocommahere"]);
        assert_eq!(split_string(""), vec![""]);
    }
    #[test]
    fn test_split_by_base_url() {
        let input_str1 = "asdfsfds http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/applications/tunnel_spec_vision_node_server";
        let expected_output1 = vec![
            "http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/applications/tunnel_spec_vision_node_server"
        ];
        let output1 = extract_substrings_containing_base_url(input_str1).unwrap();
        println!("{:?}", output1);
        assert_eq!(output1, expected_output1);
        let input_str2 = "asdfsfds http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/applications/tunnel_spec_vision_node_server fsd \
        http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/components/business/tunnel_spec_vision_node";
        let expected_output2 = vec![
            "http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/applications/tunnel_spec_vision_node_server",
            "http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/components/business/tunnel_spec_vision_node",
        ];
        let output2 = extract_substrings_containing_base_url(input_str2).unwrap();
        println!("{:?}", output2);
        assert_eq!(output2, expected_output2);
        let input_str3 = "http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/applications/tunnel_spec_vision_node_server
http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/components/business/tunnel_spec_vision_node
http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/components/rpc_client/tunnel_spec_vision_node

http://172.17.102.21:18080/svn/commondistrepo/software/auxiliary/json_util 续写";
        let output3 = extract_substrings_containing_base_url(input_str3).unwrap();
        let expected_output3 = vec![
            "http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/applications/tunnel_spec_vision_node_server",
            "http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/components/business/tunnel_spec_vision_node",
            "http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/components/rpc_client/tunnel_spec_vision_node",
        ];
        assert_eq!(output3, expected_output3);
        // http://172.17.102.22:18080/svn/softwarerepo/products/rail_transport/monitor/twds/1.x/doc/trunk 读写
        //     http://172.17.102.22:18080/svn/softwarerepo/products/rail_transport/monitor/twds/1.x/software/trunk 只读
        //     http://172.17.102.22:18080/svn/softwarerepo/products/rail_transport/monitor/twds/1.x/software/trunk/applications/debugger 读写
    }

    // #[test]
    // fn test_diff_vec() {
    //     let left: Vec<String> = fs::read_to_string("./test_data/2023-07-04-11-44-26_backup.txt")
    //         .unwrap()
    //         .lines()
    //         .map(|s| s.to_string())
    //         .collect();
    //     let right: Vec<String> = fs::read_to_string("./test_data/2023-07-04-11-44-26_new.txt")
    //         .unwrap()
    //         .lines()
    //         .map(|s| s.to_string())
    //         .collect();
    //     println!("{}", diff_slice(&left, &right));
    // }
    //
    // #[test]
    // fn test_diff_lines() {
    //     let left = fs::read_to_string("./test_data/2023-07-04-11-44-26_backup.txt")
    //         .unwrap();
    //     let right = fs::read_to_string("./test_data/2023-07-04-11-44-26_new.txt")
    //         .unwrap();
    //     let binding = prettydiff::diff_lines(&left, &right);
    //
    //     let result = binding.diff();
    //     let mut new_line: usize = 1;
    //     let mut old_line: usize = 1;
    //     for diff in &result {
    //         match diff {
    //             DiffOp::Insert(a) => {
    //                 for x in *a {
    //                     println!("line: {} + {}", new_line, x);
    //                 }
    //                 new_line += a.len();
    //             }
    //             DiffOp::Replace(a, b) => {
    //                 new_line += b.len();
    //                 old_line += a.len()
    //             }
    //             DiffOp::Remove(a) => { old_line += a.len(); }
    //             DiffOp::Equal(a) => {
    //                 new_line += a.len();
    //                 old_line += a.len()
    //             }
    //         }
    //     }
    // }

    #[test]
    fn test_convert_address() {
        let src = String::from("http://172.17.102.22:18080/svn/softwarerepo/platform/1.x/trunk/function/restful（只读）");
        let dst = convert_address(src);
        assert_eq!("[softwarerepo:/platform/1.x/trunk/function/restful]", dst);
        let src = String::from(
            "http://172.17.102.22:18080/svn/softwarerepo/platform/1.x/trunk/function/restful、只读",
        );
        let dst = convert_address(src);
        assert_eq!("[softwarerepo:/platform/1.x/trunk/function/restful]", dst);
        let src = String::from(
            "http://172.17.102.22:18080/svn/softwarerepo/platform/1.x/trunk/commonheaders(只读)",
        );
        let dst = convert_address(src);
        assert_eq!("[softwarerepo:/platform/1.x/trunk/commonheaders]", dst);
    }

    #[test]
    fn test_extract_permissions() {
        let str1 = "http://172.17.102.22:18080/svn/softwarerepo/platform/1.x/branch/1.11.0-beta-360_2023-11-29/function/robot 只读";
        assert_eq!(extract_permissions(str1), Some("只读".to_string()));
        let str1 = "http://172.17.102.22:18080/svn/softwarerepo/platform/1.x/branch/1.11.0-beta-360_2023-11-29/function/robot、读写";
        assert_eq!(extract_permissions(str1), Some("读写".to_string()));

        let str3 = r##"SVN账号名称
lizhang
注明申请路径、访问权限（只读\读写\禁止访问）
申请路径、访问权限
http://172.17.102.22:18080/svn/softwarerepo/platform/1.x/branch/1.11.0-beta-360_2023-11-29/function/robot 读写"##;
        assert_eq!(extract_permissions(str3), Some("读写".to_string()));
    }
    #[test]
    fn test_extract_account_name() {
        let str3 = r"SVN账号名称
lizhang
注明申请路径、访问权限（只读\读写\禁止访问）
申请路径、访问权限
http://172.17.102.22:18080/svn/softwarerepo/platform/1.x/branch/1.11.0-beta-360_2023-11-29/function/robot 读写";
        assert_eq!(extract_name(str3), Some("lizhang".to_string()));
    }
}
