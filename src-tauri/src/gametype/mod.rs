use crate::common::d3d11_gametype::D3D11GameType;

pub mod type_ailimit;
pub mod type_apmi;
pub mod type_doav;
pub mod type_efmi;
pub mod type_gf2;
pub mod type_gimi;
pub mod type_himi;
pub mod type_hok;
pub mod type_identityv;
pub mod type_identityv2;
pub mod type_miside;
pub mod type_naraka;
pub mod type_narakam;
pub mod type_neirr;
pub mod type_nioh2;
pub mod type_ntemi;
pub mod type_snowbreak;
pub mod type_srmi;
pub mod type_theoutcast;
pub mod type_wwmi;
pub mod type_yysls;
pub mod type_zzmi;
pub mod type_zzmidx12;

pub const GAME_NAME_LIST: &[&str] = &[
    "AILIMIT",
    "APMI",
    "DOAV",
    "EFMI",
    "GF2",
    "GIMI",
    "HIMI",
    "HOK",
    "IDENTITYV",
    "MISIDE",
    "NARAKA",
    "NARAKAM",
    "NEIRR",
    "NIOH2",
    "NTEMI",
    "SNOWBREAK",
    "SRMI",
    "THEOUTCAST",
    "WWMI",
    "YYSLS",
    "ZZMI",
    "ZZMIDX12",
];

pub fn get_game_type_list(game_name: &str) -> Result<Vec<D3D11GameType>, String> {
    match game_name.trim().to_ascii_uppercase().as_str() {
        "AILIMIT" => Ok(type_ailimit::AILIMITGameType::initialize()),
        "APMI" => Ok(type_apmi::APMIGameType::initialize()),
        "DOAV" => Ok(type_doav::DOAVGameType::initialize()),
        "EFMI" => Ok(type_efmi::EFMIGameType::initialize()),
        "GF2" => Ok(type_gf2::GF2GameType::initialize()),
        "GIMI" => Ok(type_gimi::GIMIGameType::initialize()),
        "HIMI" => Ok(type_himi::HIMIGameType::initialize()),
        "HOK" => Ok(type_hok::HOKGameType::initialize()),
        "IDENTITYV" => Ok(type_identityv2::IdentityV2GameType::initialize()),
        "MISIDE" => Ok(type_miside::MiSideGameType::initialize()),
        "NARAKA" => Ok(type_naraka::NarakaGameType::initialize()),
        "NARAKAM" => Ok(type_narakam::NarakaMGameType::initialize()),
        "NEIRR" => Ok(type_neirr::NeirRGameType::initialize()),
        "NIOH2" => Ok(type_nioh2::Nioh2GameType::initialize()),
        "NTEMI" => Ok(type_ntemi::NTEMIGameType::initialize()),
        "SNOWBREAK" => Ok(type_snowbreak::SnowBreakGameType::initialize()),
        "SRMI" => Ok(type_srmi::SRMIGameType::initialize()),
        "THEOUTCAST" => Ok(type_theoutcast::TheOutcastGameType::initialize()),
        "WUWA" => Ok(type_wwmi::WWMIGameType::initialize()),
        "WWMI" => Ok(type_wwmi::WWMIGameType::initialize()),
        "YYSLS" => Ok(type_yysls::YYSLSGameType::initialize()),
        "ZZMI" => Ok(type_zzmi::ZZMIGameType::initialize()),
        "ZZMIDX12" => Ok(type_zzmidx12::ZZMIDX12GameType::initialize()),
        other => Err(format!("Unsupported GameType preset: {}", other)),
    }
}
