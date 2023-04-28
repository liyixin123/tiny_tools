#[cfg(test)]
mod tests {
    use crate::extract_substrings_containing_base_url;


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
"http://172.17.102.22:18080/svn/softwarerepo/products/auto/inspect/tunnel_spec_vision/0.x/software/branches/baseline_0.9/components/rpc_client/tunnel_spec_vision_node"
        ];
        assert_eq!(output3, expected_output3);

    }
}