#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use druid::WidgetExt;
    use crate::{convert_address, extract_substrings_containing_base_url};
    use prettydiff::{diff_lines, diff_slice};
    use prettydiff::basic::DiffOp;
    use tokio::io::AsyncBufReadExt;


    #[test]
    fn test_split_by_base_url() {
        let input_str1 = "asdfsfds http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/applications/tunnel_spec_vision_node_server";
        let expected_output1 = vec![
            "http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/applications/tunnel_spec_vision_node_server"
        ];
        let output1 = extract_substrings_containing_base_url(input_str1);
        println!("{:?}", output1);
        assert_eq!(output1, expected_output1);
        let input_str2 = "asdfsfds http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/applications/tunnel_spec_vision_node_server fsd \
        http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/components/business/tunnel_spec_vision_node";
        let expected_output2 = vec![
            "http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/applications/tunnel_spec_vision_node_server",
            "http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/components/business/tunnel_spec_vision_node",
        ];
        let output2 = extract_substrings_containing_base_url(input_str2);
        println!("{:?}", output2);
        assert_eq!(output2, expected_output2);
        let input_str3 = "http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/applications/tunnel_spec_vision_node_server
http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/components/business/tunnel_spec_vision_node
http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/components/rpc_client/tunnel_spec_vision_node

http://172.17.102.21:18080/svn/commondistrepo/software/auxiliary/json_util 续写";
        let output3 = extract_substrings_containing_base_url(input_str3);
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

    #[test]
    fn test_diff_vec() {
        let left: Vec<String> = fs::read_to_string("C:/Users/liyixin/AppData/Roaming/svn_user_auth/backups/2023-07-04-11-44-26_backup.txt")
            .unwrap()
            .lines()
            .map(|s| s.to_string())
            .collect();
        let right: Vec<String> = fs::read_to_string("C:/Users/liyixin/AppData/Roaming/svn_user_auth/backups/2023-07-04-11-44-26_new.txt")
            .unwrap()
            .lines()
            .map(|s| s.to_string())
            .collect();
        println!("{}", diff_slice(&left, &right));
    }

    #[test]
    fn test_diff_lines() {
        let left = fs::read_to_string("C:/Users/liyixin/AppData/Roaming/svn_user_auth/backups/2023-07-04-11-44-26_backup.txt")
            .unwrap();
        let right = fs::read_to_string("C:/Users/liyixin/AppData/Roaming/svn_user_auth/backups/2023-07-04-11-44-26_new.txt")
            .unwrap();
        let binding = prettydiff::diff_lines(&left, &right);

        let result = binding.diff();
        let mut new_line: usize = 1;
        let mut old_line: usize = 1;
        for diff in &result {
            match diff {
                DiffOp::Insert(a) => {
                    for x in *a {
                        println!("line: {} + {}", new_line, x);
                    }
                    new_line += a.len();
                }
                DiffOp::Replace(a, b) => {
                    new_line += b.len();
                    old_line += a.len()
                }
                DiffOp::Remove(a) => { old_line += a.len(); }
                DiffOp::Equal(a) => {
                    new_line += a.len();
                    old_line += a.len()
                }
            }
        }
    }

    #[test]
    fn test_convert_address() {
        let src = String::from("http://172.17.102.22:18080/svn/softwarerepo/platform/1.x/trunk/function/restful（只读）");
        let dst = convert_address(src);
        assert_eq!("[softwarerepo:/platform/1.x/trunk/function/restful]", dst);
        let src = String::from("http://172.17.102.22:18080/svn/softwarerepo/platform/1.x/trunk/function/restful、只读");
        let dst = convert_address(src);
        assert_eq!("[softwarerepo:/platform/1.x/trunk/function/restful]", dst);
    }
}