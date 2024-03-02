
// Generated from didc
// TODO: move declarations into a separate module
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Deserialize, Principal, Encode, Decode};
use ic_cdk::api::call::CallResult ;



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
    pub async fn add_wizard(&self, arg0: WizardDetails) -> CallResult<(Response,)> {
        ic_cdk::call(self.0, "addWizard", (arg0,)).await
    }
    pub async fn delete_wizard(&self, arg0: String) -> CallResult<(Response,)> {
        ic_cdk::call(self.0, "deleteWizard", (arg0,)).await
    }
    pub async fn get_all_analytics(&self) -> CallResult<(Vec<(String, Analytics)>,)> {
        ic_cdk::call(self.0, "getAllAnalytics", ()).await
    }
    pub async fn get_all_wizards(&self) -> CallResult<(Vec<WizardDetails>,)> {
        ic_cdk::call(self.0, "getAllWizards", ()).await
    }
    pub async fn get_analytics(&self, arg0: String) -> CallResult<(AnalyticsV1,)> {
        ic_cdk::call(self.0, "getAnalytics", (arg0,)).await
    }
    pub async fn get_user_wizards(&self, arg0: String) -> CallResult<(Vec<WizardDetailsBasic>,)> {
        ic_cdk::call(self.0, "getUserWizards", (arg0,)).await
    }
    pub async fn get_wizard(&self, arg0: String) -> CallResult<(Option<WizardDetails>,)> {
        ic_cdk::call(self.0, "getWizard", (arg0,)).await
    }
    pub async fn get_wizards(&self) -> CallResult<(Vec<WizardDetailsBasic>,)> {
        ic_cdk::call(self.0, "getWizards", ()).await
    }
    pub async fn is_wizard_name_valid(&self, arg0: String) -> CallResult<(bool,)> {
        ic_cdk::call(self.0, "isWizardNameValid", (arg0,)).await
    }
    pub async fn publish_wizard(&self, arg0: String) -> CallResult<(Response,)> {
        ic_cdk::call(self.0, "publishWizard", (arg0,)).await
    }
    pub async fn unpublish_wizard(&self, arg0: String) -> CallResult<(Response,)> {
        ic_cdk::call(self.0, "unpublishWizard", (arg0,)).await
    }
    pub async fn update_message_analytics(&self, arg0: String) -> CallResult<()> {
        ic_cdk::call(self.0, "updateMessageAnalytics", (arg0,)).await
    }
}
