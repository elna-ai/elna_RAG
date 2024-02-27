// use candid::{CandidType, Principal};
// use ic_cdk::api::call::CallResult;
// use serde::Deserialize;

// #![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::call::CallResult as Result;

pub async fn get_agent_details(wizard_id: String) -> Option<WizardDetails> {
    let wizard_details_service =
        Service(Principal::from_text("gichg-2iaaa-aaaah-adtia-cai").unwrap());
    let result = wizard_details_service.get_wizard(wizard_id).await;
    let a = wizard_details_service
        .delete_wizard(String::from("arg0"))
        .await;
    match result {
        Ok((wizard_details,)) => wizard_details,
        _ => None,
    }
}

#[derive(CandidType, Deserialize)]
pub enum WizardVisibility {
    #[serde(rename = "privateVisibility")]
    PrivateVisibility,
    #[serde(rename = "publicVisibility")]
    PublicVisibility,
    #[serde(rename = "unlistedVisibility")]
    UnlistedVisibility,
}

#[derive(CandidType, Deserialize)]
pub struct WizardDetails {
    pub id: String,
    pub isPublished: bool,
    pub userId: String,
    pub name: String,
    pub biography: String,
    pub greeting: String,
    pub description: String,
    pub summary: Option<String>,
    pub visibility: WizardVisibility,
    pub avatar: String,
}

#[derive(CandidType, Deserialize)]
pub struct Response {
    pub status: candid::Nat,
    pub message: String,
}

#[derive(CandidType, Deserialize)]
pub struct AnalyticsV1 {
    pub messagesReplied: candid::Nat,
}

#[derive(CandidType, Deserialize)]
pub enum Analytics {
    #[serde(rename = "v1")]
    V1(AnalyticsV1),
}

#[derive(CandidType, Deserialize)]
pub struct WizardDetailsBasic {
    pub id: String,
    pub isPublished: bool,
    pub userId: String,
    pub name: String,
    pub biography: String,
    pub description: String,
    pub avatar: String,
}

candid::define_service!(pub Main : {
  "addWizard" : candid::func!((WizardDetails) -> (Response));
  "deleteWizard" : candid::func!((String) -> (Response));
  "getAllAnalytics" : candid::func!(() -> (Vec<(String,Analytics,)>) query);
  "getAllWizards" : candid::func!(() -> (Vec<WizardDetails>));
  "getAnalytics" : candid::func!((String) -> (AnalyticsV1) query);
  "getUserWizards" : candid::func!((String) -> (Vec<WizardDetailsBasic>) query);
  "getWizard" : candid::func!((String) -> (Option<WizardDetails>) query);
  "getWizards" : candid::func!(() -> (Vec<WizardDetailsBasic>) query);
  "isWizardNameValid" : candid::func!((String) -> (bool) query);
  "publishWizard" : candid::func!((String) -> (Response));
  "unpublishWizard" : candid::func!((String) -> (Response));
  "updateMessageAnalytics" : candid::func!((String) -> ());
});
pub struct Service(pub Principal);
impl Service {
    pub async fn add_wizard(&self, arg0: WizardDetails) -> Result<(Response,)> {
        ic_cdk::call(self.0, "addWizard", (arg0,)).await
    }
    pub async fn delete_wizard(&self, arg0: String) -> Result<(Response,)> {
        ic_cdk::call(self.0, "deleteWizard", (arg0,)).await
    }
    pub async fn get_all_analytics(&self) -> Result<(Vec<(String, Analytics)>,)> {
        ic_cdk::call(self.0, "getAllAnalytics", ()).await
    }
    pub async fn get_all_wizards(&self) -> Result<(Vec<WizardDetails>,)> {
        ic_cdk::call(self.0, "getAllWizards", ()).await
    }
    pub async fn get_analytics(&self, arg0: String) -> Result<(AnalyticsV1,)> {
        ic_cdk::call(self.0, "getAnalytics", (arg0,)).await
    }
    pub async fn get_user_wizards(&self, arg0: String) -> Result<(Vec<WizardDetailsBasic>,)> {
        ic_cdk::call(self.0, "getUserWizards", (arg0,)).await
    }
    pub async fn get_wizard(&self, arg0: String) -> Result<(Option<WizardDetails>,)> {
        ic_cdk::call(self.0, "getWizard", (arg0,)).await
    }
    pub async fn get_wizards(&self) -> Result<(Vec<WizardDetailsBasic>,)> {
        ic_cdk::call(self.0, "getWizards", ()).await
    }
    pub async fn is_wizard_name_valid(&self, arg0: String) -> Result<(bool,)> {
        ic_cdk::call(self.0, "isWizardNameValid", (arg0,)).await
    }
    pub async fn publish_wizard(&self, arg0: String) -> Result<(Response,)> {
        ic_cdk::call(self.0, "publishWizard", (arg0,)).await
    }
    pub async fn unpublish_wizard(&self, arg0: String) -> Result<(Response,)> {
        ic_cdk::call(self.0, "unpublishWizard", (arg0,)).await
    }
    pub async fn update_message_analytics(&self, arg0: String) -> Result<()> {
        ic_cdk::call(self.0, "updateMessageAnalytics", (arg0,)).await
    }
}

// #[derive(Debug, Deserialize, CandidType)]
// enum WizardVisibility {
//     PrivateVisibility,
//     PublicVisibility,
//     UnlistedVisibility,
// }

// #[derive(Debug, Deserialize, CandidType)]
// pub struct WizardDetails {
//     avatar: String,
//     biography: String,
//     description: String,
//     greeting: String,
//     id: String,
//     is_published: bool,
//     name: String,
//     summary: Option<String>,
//     user_id: String,
//     visibility: WizardVisibility,
// }

// trait WizardDetailsService {
//     fn get_wizard(text: &str) -> Option<WizardDetails>;
// }
