use serde::Deserialize;
use serde_repr::Deserialize_repr;
use std::collections::HashMap;

pub mod player {
    use super::*;

    /// url: https://enka.network/api/uid/{uid}/
    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        pub avatar_info_list: Option<Vec<AvatarInfo>>,
        #[serde(flatten)]
        pub info: player::info::Info,
    }

    pub mod info {
        use super::*;

        /// url: https://enka.network/api/uid/{uid}?info
        #[derive(Deserialize, Debug)]
        pub struct Response(pub Info);

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct Info {
            pub player_info: PlayerInfo,
            pub ttl: u64,
            pub uid: String, // WHY
            pub owner: Option<Owner>,
        }

        #[derive(Deserialize, Debug)]
        pub struct Owner {
            pub hash: profile::hoyo::Hash,
            #[serde(flatten)]
            pub info: profile::info::Info,
        }
    }
}

pub mod profile {
    use super::*;

    pub mod info {
        use super::*;

        /// url: https://enka.network/api/profile/{owner.username}/?format=json
        #[derive(Deserialize, Debug)]
        pub struct Response(pub Info);

        #[derive(Deserialize, Debug)]
        #[serde(deny_unknown_fields)]
        pub struct Info {
            pub username: String,
            pub profile: Profile,
            pub id: u64,
        }

        #[derive(Deserialize, Debug)]
        #[serde(deny_unknown_fields)]
        pub struct Profile {
            pub bio: String,
            pub level: i64,
            pub signup_state: Option<u8>, // stopped appearing for unknown reasons
            pub avatar: Option<String>,
            pub image_url: Option<String>, // Patreon image
        }
    }

    pub mod hoyos {
        use super::*;

        /// url: https://enka.network/api/profile/{owner.username}/hoyos/
        #[derive(Deserialize, Debug)]
        pub struct Response(pub HashMap<hoyo::Hash, hoyo::Hoyo>);
    }

    pub mod hoyo {
        use super::*;
        pub type Hash = String;

        /// url: https://enka.network/api/profile/{owner.username}/hoyos/{owner.hash}/?format=json
        #[derive(Deserialize, Debug)]
        pub struct Response(pub Hoyo);

        #[allow(clippy::large_enum_variant)]
        #[derive(/* Deserialize, */ Debug)]
        // #[serde(tag = "hoyo_type")]
        pub enum Hoyo {
            // #[serde(rename = 1)]
            Genshin(GenshinHoyo),
            // #[serde(other)]
            // Other,
            Other(serde_json::Value),
        }

        // HACK: necessary due to https://github.com/serde-rs/serde/issues/745
        impl<'de> Deserialize<'de> for Hoyo {
            fn deserialize<D: serde::de::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                serde_json::Value::deserialize(d).and_then(|v| match v["hoyo_type"].as_u64() {
                    Some(0) => serde_json::from_value(v)
                        .map(Hoyo::Genshin)
                        .map_err(serde::de::Error::custom),
                    Some(1 | 2) => Ok(Hoyo::Other(v)),
                    _ => Err(serde::de::Error::custom("unknown Hoyo variant")),
                })
            }
        }

        #[derive(Deserialize, Debug)]
        #[serde(deny_unknown_fields)]
        pub struct GenshinHoyo {
            pub uid: Option<u64>,
            pub uid_public: bool,
            pub public: bool,
            pub live_public: bool,
            pub verified: bool,
            pub player_info: PlayerInfo,
            pub hash: Hash,
            pub region: Region,
            pub order: u64,
            pub avatar_order: Option<HashMap<AvatarId, u64>>,
            pub hoyo_type: u8, // TODO: check, HoyoKind I assume 0 gi, 1 hsr, 2 zzz
        }

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "UPPERCASE")]
        pub enum Region {
            #[serde(rename = "")]
            Internal, // 0
            #[serde(rename = "CN")]
            Celestia, // 1, 2, 3
            #[serde(rename = "B")]
            Irminsul, // 5
            #[serde(rename = "NA")]
            America, // 6
            #[serde(rename = "EU")]
            Europe, // 7
            Asia, // 8, 18
            #[serde(rename = "TW")]
            TaiwanHonKongMacao, // 9
        }

        pub mod builds {
            use super::*;

            /// url: https://enka.network/api/profile/{owner.username}/hoyos/{owner.hash}/builds/
            #[derive(Deserialize, Debug)]
            pub struct Response(pub HashMap<AvatarId, Vec<build::Build>>);
        }

        pub mod build {
            use super::*;

            /// url: https://enka.network/api/profile/{owner.username}/hoyos/{owner.hash}/builds/{build.id}
            #[derive(Deserialize, Debug)]
            pub struct Response(pub Build);

            #[derive(Deserialize, Debug)]
            #[serde(deny_unknown_fields)]
            pub struct Build {
                pub id: u64,
                pub name: String,
                pub avatar_id: String, // this is an AvatarId as a String
                pub avatar_data: AvatarInfo,
                pub order: u64,
                pub live: bool,
                pub settings: Settings,
                pub public: bool,
                pub image: Option<String>,
                pub hoyo_type: u8,
                pub hoyo: Hash,
            }

            #[derive(Deserialize, Debug)]
            #[serde(rename_all = "camelCase", deny_unknown_fields)]
            pub struct Settings {
                pub adaptive_color: Option<bool>,
                pub art_source: Option<String>,
                pub caption: Option<String>,
                pub honkard_width: Option<f64>,
                pub transform: Option<serde_json::Value>,
            }
        }
    }
}

pub type AvatarId = u64;
pub type CostumeId = u64;
pub type ItemId = u64;
pub type NameCardId = u64;
pub type ProfilePictureId = u64;
pub type SkillId = u64; // TODO: check, may be TalentId
pub type TalentId = u64;

#[derive(Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(untagged)]
pub enum TextMapHash {
    String(String),
    U64(u64), // old builds may still use u64 in db
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerInfo {
    pub nickname: String,
    pub level: u8,
    pub signature: Option<String>,
    pub world_level: Option<u8>,
    pub name_card_id: NameCardId,
    pub finish_achievement_num: u64,
    pub tower_floor_index: Option<u8>,
    pub tower_level_index: Option<u8>,
    pub tower_star_index: Option<u8>,
    pub theater_mode_index: Option<u8>,
    pub theater_act_index: Option<u8>,
    pub theater_star_index: Option<u8>,
    pub is_show_avatar_talent: Option<bool>,
    pub show_avatar_info_list: Option<Vec<ShowAvatarInfo>>,
    pub show_name_card_id_list: Option<Vec<NameCardId>>,
    pub profile_picture: ProfilePicture,
    pub fetter_count: Option<u8>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum ProfilePicture {
    AvatarId(AvatarId),
    Id(ProfilePictureId),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShowAvatarInfo {
    pub avatar_id: AvatarId,
    pub level: u8,
    pub energy_type: Option<u8>,
    pub costume_id: Option<CostumeId>,
    pub talent_level: Option<u8>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AvatarInfo {
    pub avatar_id: AvatarId,
    pub prop_map: HashMap<Prop, PropMap>,
    pub talent_id_list: Option<Vec<TalentId>>,
    pub fight_prop_map: HashMap<u32, f64>,
    pub skill_depot_id: SkillId,
    pub inherent_proud_skill_list: Vec<SkillId>,
    pub skill_level_map: HashMap<u64, u64>,
    pub proud_skill_extra_level_map: Option<HashMap<u64, u64>>,
    pub equip_list: Vec<Equip>,
    pub fetter_info: Option<AvatarInfoFetterInfo>,
    pub costume_id: Option<CostumeId>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AvatarInfoFetterInfo {
    pub exp_level: u8,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PropMap {
    pub r#type: Prop,
    pub ival: Option<String>,
    pub val: Option<String>,
}

#[non_exhaustive]
#[derive(Deserialize_repr, Debug, Clone, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Prop {
    // TODO: choose nomenclature
    // PROP_NONE = 0,
    // PROP_EXP = 1001,
    Xp = 1001,
    // PROP_BREAK_LEVEL = 1002,
    Ascension = 1002,
    // PROP_SATIATION_VAL = 1003,
    Unknown = 1003,
    // PROP_SATIATION_PENALTY_TIME = 1004,
    Unknown2 = 1004,
    // PROP_GEAR_START_VAL = 2001,
    // PROP_GEAR_STOP_VAL = 2002,
    // PROP_LEVEL = 4001,
    Level = 4001,
    // PROP_LAST_CHANGE_AVATAR_TIME = 10001,
    // PROP_MAX_SPRING_VOLUME = 10002,
    // PROP_CUR_SPRING_VOLUME = 10003,
    // PROP_IS_SPRING_AUTO_USE = 10004,
    // PROP_SPRING_AUTO_USE_PERCENT = 10005,
    // PROP_IS_FLYABLE = 10006,
    // PROP_IS_WEATHER_LOCKED = 10007,
    // PROP_IS_GAME_TIME_LOCKED = 10008,
    // PROP_IS_TRANSFERABLE = 10009,
    // PROP_MAX_STAMINA = 10010,
    Unknown3 = 10010,
    // PROP_CUR_PERSIST_STAMINA = 10011,
    // PROP_CUR_TEMPORARY_STAMINA = 10012,
    // PROP_PLAYER_LEVEL = 10013,
    // PROP_PLAYER_EXP = 10014,
    // PROP_PLAYER_HCOIN = 10015,
    // PROP_PLAYER_SCOIN = 10016,
    // PROP_PLAYER_MP_SETTING_TYPE = 10017,
    // PROP_IS_MP_MODE_AVAILABLE = 10018,
    // PROP_PLAYER_WORLD_LEVEL = 10019,
    // PROP_PLAYER_RESIN = 10020,
    // PROP_PLAYER_WAIT_SUB_HCOIN = 10022,
    // PROP_PLAYER_WAIT_SUB_SCOIN = 10023,
    // PROP_IS_ONLY_MP_WITH_PS_PLAYER = 10024,
    // PROP_PLAYER_MCOIN = 10025,
    // PROP_PLAYER_WAIT_SUB_MCOIN = 10026,
    // PROP_PLAYER_LEGENDARY_KEY = 10027,
    // PROP_IS_HAS_FIRST_SHARE = 10028,
    // PROP_PLAYER_FORGE_POINT = 10029,
    // PROP_CUR_CLIMATE_METER = 10035,
    // PROP_CUR_CLIMATE_TYPE = 10036,
    // PROP_CUR_CLIMATE_AREA_ID = 10037,
    // PROP_CUR_CLIMATE_AREA_CLIMATE_TYPE = 10038,
    // PROP_PLAYER_WORLD_LEVEL_LIMIT = 10039,
    // PROP_PLAYER_WORLD_LEVEL_ADJUST_CD = 10040,
    // PROP_PLAYER_LEGENDARY_DAILY_TASK_NUM = 10041,
    // PROP_PLAYER_HOME_COIN = 10042,
    // PROP_PLAYER_WAIT_SUB_HOME_COIN = 10043,
    // PROP_IS_AUTO_UNLOCK_SPECIFIC_EQUIP = 10044,
    // PROP_PLAYER_GCG_COIN = 10045,
    // PROP_PLAYER_WAIT_SUB_GCG_COIN = 10046,
    // PROP_PLAYER_ONLINE_TIME = 10047,
    // PROP_IS_DIVEABLE = 10048,
    // PROP_MAX_DIVE_STAMINA = 10049,
    Unknown4 = 10049,
    // PROP_CUR_PERSIST_DIVE_STAMINA = 10050
}

#[derive(/* Deserialize, */ Debug, Clone)]
// #[serde(untagged)]
pub enum Equip {
    Weapon(EquipWeapon),
    Reliquary(EquipReliquary),
}

// HACK: necessary due to https://github.com/serde-rs/json/issues/1103
impl<'de> Deserialize<'de> for Equip {
    fn deserialize<D: serde::de::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        serde_json::Value::deserialize(d).and_then(|v| match &v["flat"]["itemType"] {
            serde_json::Value::String(s) if s == "ITEM_RELIQUARY" => serde_json::from_value(v)
                .map(Equip::Reliquary)
                .map_err(serde::de::Error::custom),
            serde_json::Value::String(s) if s == "ITEM_WEAPON" => serde_json::from_value(v)
                .map(Equip::Weapon)
                .map_err(serde::de::Error::custom),
            _ => Err(serde::de::Error::custom("unknown Equip variant")),
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EquipWeapon {
    pub item_id: ItemId,
    pub weapon: Weapon,
    pub flat: FlatWeapon,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EquipReliquary {
    pub item_id: ItemId,
    pub reliquary: Reliquary,
    pub flat: FlatReliquary,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Weapon {
    pub level: u8,
    pub promote_level: Option<u8>,
    pub affix_map: Option<HashMap<u64, u64>>, // see equip.rs
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Reliquary {
    pub level: u8,
    pub exp: Option<u64>,
    pub main_prop_id: MainPropId,
    pub append_prop_id_list: Option<Vec<AppendPropId>>,
}

type MainPropId = u32;
type AppendPropId = u32;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlatWeapon {
    pub name_text_map_hash: TextMapHash,
    pub rank_level: u8,
    pub item_type: String,
    pub icon: String,
    pub weapon_stats: Vec<WeaponStat>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlatReliquary {
    pub name_text_map_hash: TextMapHash,
    pub set_name_text_map_hash: TextMapHash,
    pub rank_level: u8,
    pub reliquary_mainstat: MainStat,
    pub reliquary_substats: Option<Vec<SubStat>>,
    pub item_type: String,
    pub icon: String,
    pub equip_type: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EquipType {
    EquipBracer,
    EquipNeackle,
    EquipShoes,
    EquipRing,
    EquipDress,
}

type GameStat = String; // TODO: unimplemented
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MainStat {
    pub main_prop_id: GameStat,
    pub stat_value: f64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubStat {
    pub append_prop_id: GameStat,
    pub stat_value: f64,
}
type WeaponStat = SubStat;
