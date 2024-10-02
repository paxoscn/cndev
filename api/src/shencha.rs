
use alibaba_cloud_sdk_rust::sdk::requests::BaseRequestExt;
use alibaba_cloud_sdk_rust::sdk::requests::{self, BaseRequest};

use serde::{Deserialize, Serialize};

#[derive(Default, Debug)]
pub struct ShenchaRequest {
    rpcRequest: alibaba_cloud_sdk_rust::sdk::requests::RpcRequest,
    pub Service: String,
    pub ServiceParameters: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ShenchaRequestServiceParameters {
    pub content: String,
}

impl BaseRequestExt for ShenchaRequest {
    fn base(&self) -> &BaseRequest {
        self.rpcRequest.base()
    }

    fn base_as_mut(&mut self) -> &mut BaseRequest {
        self.rpcRequest.base_as_mut()
    }
}

impl ShenchaRequest {
    pub fn BuildQueryParams(&mut self) {
        self.addQueryParam("Service", &self.Service.to_owned());
        self.addQueryParam("ServiceParameters", &self.ServiceParameters.to_owned());
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ShenchasResponse {
    // baseResponse: responses::BaseResponse,
    pub RequestId: String, //`json:"RequestId" xml:"RequestId"`
    pub Code: i32,      //`json:"Code" xml:"Code"`
    pub Msg: Option<String>,
    pub Message: Option<String>,
    pub Data: Option<ShenchasResponseData>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ShenchasResponseData {
    pub reason: String,
    pub labels: String,
}

pub fn CreateShenchaRequest() -> ShenchaRequest {
    let mut request = ShenchaRequest::default();
    // Find the version at https://help.aliyun.com/document_detail/467828.html
    // or https://next.api.aliyun.com/api/Green/2022-03-02/TextModeration?lang=JAVA&RegionId=cn-beijing&tab=CLI
    request
        .rpcRequest
        .InitWithApiInfo("", "2022-03-02", "TextModeration", "", "");
    request.SetMethod(requests::POST);
    request.SetDomain("green-cip.cn-beijing.aliyuncs.com");
    request
}

pub fn shencha(
    aliyun_client: &mut alibaba_cloud_sdk_rust::services::dysmsapi::Client,
    service_type: &str,
    content: &str
) -> Result<bool, std::io::Error> {
    let mut request = CreateShenchaRequest();
    request.Service = String::from(service_type);

    let mut service_parameters = ShenchaRequestServiceParameters::default();
    service_parameters.content = String::from(content);
    request.ServiceParameters = serde_json::to_string(&service_parameters).unwrap();

    request.BuildQueryParams();
    
    let mut response = ShenchasResponse::default();
    let mut baseResponse = alibaba_cloud_sdk_rust::sdk::responses::BaseResponse::default();

    aliyun_client.DoAction(&mut request.rpcRequest, &mut baseResponse)?;

    let res_str = String::from_utf8(baseResponse.httpContentBytes).unwrap();
    response = serde_json::from_str(res_str.as_str())?;

    match response.Code {
        200 => {
            match response.Data {
                Some(data) => {
                    if data.reason.len() < 1 {
                        Ok(true)
                    } else {
                        println!("failed res = {}", res_str);

                        Ok(false)
                    }
                },
                None => {
                    println!("failed res = {}", res_str);

                    Ok(false)
                }
            }
        }
        _ => {
            println!("failed res = {}", res_str);

            Ok(false)
        }
    }
}