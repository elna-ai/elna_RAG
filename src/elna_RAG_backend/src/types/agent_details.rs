// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::call::CallResult as Result;

#[derive(CandidType, Deserialize)]
pub struct InitialArgs {
    pub capCanisterId: Principal,
    pub owner: Principal,
    pub userManagementCanisterId: Principal,
    pub ragCanisterId: Principal,
    pub elnaImagesCanisterId: Principal,
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
pub struct WizardDetailsV3 {
    pub id: String,
    pub isPublished: bool,
    pub tokenAddress: Option<String>,
    pub userId: String,
    pub name: String,
    pub biography: String,
    pub greeting: String,
    pub description: String,
    pub summary: Option<String>,
    pub poolAddress: Option<String>,
    pub visibility: WizardVisibility,
    pub avatar: String,
}

#[derive(CandidType, Deserialize)]
pub enum Error {
    UnableToUploadAvatar,
    PrincipalIdMissMatch,
    AgentNotFound,
    AgentIdExist,
    UserNotAuthorized,
}

#[derive(CandidType, Deserialize)]
pub enum Result_ {
    #[serde(rename = "ok")]
    Ok(String),
    #[serde(rename = "err")]
    Err(Error),
}

#[derive(CandidType, Deserialize)]
pub struct AnalyticsV2External {
    pub modificationCount: candid::Nat,
    pub messagesReplied: candid::Nat,
    pub uniqueUsers: candid::Nat,
}

pub type Time = candid::Int;
#[derive(CandidType, Deserialize)]
pub struct WizardDetailsWithTimeStamp {
    pub id: String,
    pub isPublished: bool,
    pub userId: String,
    pub name: String,
    pub createdAt: Time,
    pub biography: String,
    pub greeting: String,
    pub description: String,
    pub summary: Option<String>,
    pub updatedAt: Time,
    pub visibility: WizardVisibility,
    pub avatar: String,
}

#[derive(CandidType, Deserialize)]
pub struct WizardDetailsBasicWithCreatorName {
    pub id: String,
    pub isPublished: bool,
    pub tokenAddress: Option<String>,
    pub userId: String,
    pub name: String,
    pub createdAt: Time,
    pub biography: String,
    pub description: String,
    pub creatorName: String,
    pub updatedAt: Time,
    pub poolAddress: Option<String>,
    pub avatar: String,
}

#[derive(CandidType, Deserialize)]
pub struct WizardUpdateDetails {
    pub name: String,
    pub biography: String,
    pub greeting: String,
    pub description: String,
    pub visibility: WizardVisibility,
    pub avatar: String,
}

#[derive(CandidType, Deserialize)]
pub struct MainUpdateWizardLaunchpadArg1 {
    pub tokenAddress: Option<String>,
    pub userId: String,
    pub agentId: String,
    pub poolAddress: Option<String>,
}

candid::define_service!(pub Main : {
  "addWizard" : candid::func!((WizardDetails) -> (Response));
  "addWizardLaunchpad" : candid::func!((WizardDetailsV3) -> (Result_));
  "deleteWizard" : candid::func!((String) -> (Response));
  "getAllAnalytics" : candid::func!(
    () -> (Vec<(String,AnalyticsV2External,)>) query
  );
  "getAllWizards" : candid::func!(() -> (Vec<WizardDetailsWithTimeStamp>));
  "getAnalytics" : candid::func!((String) -> (AnalyticsV2External) query);
  "getElnaBackendUri" : candid::func!(() -> (String) query);
  "getLaunchpadOwner" : candid::func!(() -> (String) query);
  "getUserWizards" : candid::func!(
    (String) -> (Vec<WizardDetailsBasicWithCreatorName>)
  );
  "getWizard" : candid::func!((String) -> (Option<WizardDetailsV3>) query);
  "getWizards" : candid::func!(() -> (Vec<WizardDetailsBasicWithCreatorName>));
  "isWizardNameValid" : candid::func!((String) -> (bool) query);
  "publishWizard" : candid::func!((String) -> (Response));
  "unpublishWizard" : candid::func!((String) -> (Response));
  "updateKnowledgeAnalytics" : candid::func!((String) -> (String));
  "updateLaunchpadOwner" : candid::func!((Principal) -> (String));
  "updateMessageAnalytics" : candid::func!((String) -> ());
  "updateWizard" : candid::func!((String, WizardUpdateDetails) -> (String));
  "updateWizardAdmin" : candid::func!((String, String) -> (String));
  "updateWizardLaunchpad" : candid::func!(
    (String, MainUpdateWizardLaunchpadArg1) -> (Result_)
  );
});
pub struct Service(pub Principal);
impl Service {
    pub async fn add_wizard(&self, arg0: WizardDetails) -> Result<(Response,)> {
        ic_cdk::call(self.0, "addWizard", (arg0,)).await
    }
    pub async fn add_wizard_launchpad(&self, arg0: WizardDetailsV3) -> Result<(Result_,)> {
        ic_cdk::call(self.0, "addWizardLaunchpad", (arg0,)).await
    }
    pub async fn delete_wizard(&self, arg0: String) -> Result<(Response,)> {
        ic_cdk::call(self.0, "deleteWizard", (arg0,)).await
    }
    pub async fn get_all_analytics(&self) -> Result<(Vec<(String, AnalyticsV2External)>,)> {
        ic_cdk::call(self.0, "getAllAnalytics", ()).await
    }
    pub async fn get_all_wizards(&self) -> Result<(Vec<WizardDetailsWithTimeStamp>,)> {
        ic_cdk::call(self.0, "getAllWizards", ()).await
    }
    pub async fn get_analytics(&self, arg0: String) -> Result<(AnalyticsV2External,)> {
        ic_cdk::call(self.0, "getAnalytics", (arg0,)).await
    }
    pub async fn get_elna_backend_uri(&self) -> Result<(String,)> {
        ic_cdk::call(self.0, "getElnaBackendUri", ()).await
    }
    pub async fn get_launchpad_owner(&self) -> Result<(String,)> {
        ic_cdk::call(self.0, "getLaunchpadOwner", ()).await
    }
    pub async fn get_user_wizards(
        &self,
        arg0: String,
    ) -> Result<(Vec<WizardDetailsBasicWithCreatorName>,)> {
        ic_cdk::call(self.0, "getUserWizards", (arg0,)).await
    }
    pub async fn get_wizard(&self, arg0: String) -> Result<(Option<WizardDetailsV3>,)> {
        ic_cdk::call(self.0, "getWizard", (arg0,)).await
    }
    pub async fn get_wizards(&self) -> Result<(Vec<WizardDetailsBasicWithCreatorName>,)> {
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
    pub async fn update_knowledge_analytics(&self, arg0: String) -> Result<(String,)> {
        ic_cdk::call(self.0, "updateKnowledgeAnalytics", (arg0,)).await
    }
    pub async fn update_launchpad_owner(&self, arg0: Principal) -> Result<(String,)> {
        ic_cdk::call(self.0, "updateLaunchpadOwner", (arg0,)).await
    }
    pub async fn update_message_analytics(&self, arg0: String) -> Result<()> {
        ic_cdk::call(self.0, "updateMessageAnalytics", (arg0,)).await
    }
    pub async fn update_wizard(
        &self,
        arg0: String,
        arg1: WizardUpdateDetails,
    ) -> Result<(String,)> {
        ic_cdk::call(self.0, "updateWizard", (arg0, arg1)).await
    }
    pub async fn update_wizard_admin(&self, arg0: String, arg1: String) -> Result<(String,)> {
        ic_cdk::call(self.0, "updateWizardAdmin", (arg0, arg1)).await
    }
    pub async fn update_wizard_launchpad(
        &self,
        arg0: String,
        arg1: MainUpdateWizardLaunchpadArg1,
    ) -> Result<(Result_,)> {
        ic_cdk::call(self.0, "updateWizardLaunchpad", (arg0, arg1)).await
    }
}
