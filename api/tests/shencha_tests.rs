#[cfg(test)]
mod shencha_tests {
    use super::*;
    use cndev_api::shencha;
    use alibaba_cloud_sdk_rust::services::dysmsapi::Client;
    // use mockall::predicate::*;
    // use mockall::mock;

    // mock! {
    //     SmsClient {}
    //     impl Client for SmsClient {
    //         fn DoAction(&mut self, request: &mut dyn alibaba_cloud_sdk_rust::sdk::requests::AcsRequest, response: &mut dyn alibaba_cloud_sdk_rust::sdk::responses::AcsResponse) -> Result<(), std::io::Error>;
    //     }
    // }

    #[test]
    fn test_shencha() {
        let aliyun_sms_region = "cn-beijing";
        let aliyun_sms_ak = "";
        let aliyun_sms_sk = "";
    
        let mut aliyun_sms_client = alibaba_cloud_sdk_rust::services::dysmsapi::Client::NewClientWithAccessKey(
            aliyun_sms_region,
            aliyun_sms_ak,
            aliyun_sms_sk,
        ).unwrap();

        let result = shencha::shencha(aliyun_sms_client, "nickname_detection", "色情");
        
        match result {
            Ok(true) => {
                assert!(false);
            }
            Ok(false) => {
                assert!(true);
            }
            Err(e) => {
                println!("!!! {:?}", e);
            }
        }
        
        // assert!(result.is_ok());

        // let mut mock_client = MockSmsClient::new();
        // mock_client.expect_DoAction()
        //     .returning(|_, _| Ok(()));

        // let result = shencha(mock_client, "test_service", "test_content");
        // assert!(result.is_ok());
        // assert_eq!(result.unwrap(), true);
    }
}