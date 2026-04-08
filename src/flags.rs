/// Returns a Unicode flag emoji for a team by its three-letter abbreviation (TLA).
///
/// UK constituent nations (ENG, SCO, WAL, NIR) use tag-sequence emoji. All
/// other known football nations map through an ISO alpha-3 → alpha-2 lookup,
/// then to a regional indicator pair. Unknown or missing TLAs fall back to 🏳.
pub fn flag_emoji(tla: Option<&str>) -> String {
    let Some(tla) = tla else {
        return "🏳".to_string();
    };
    // Tag-sequence emoji for UK constituent nations
    match tla {
        "ENG" => return "🏴󠁧󠁢󠁥󠁮󠁧󠁿".to_string(),
        "SCO" => return "🏴󠁧󠁢󠁳󠁣󠁴󠁿".to_string(),
        "WAL" => return "🏴󠁧󠁢󠁷󠁬󠁳󠁿".to_string(),
        "NIR" => return "🏴󠁧󠁢󠁮󠁩󠁲󠁿".to_string(),
        _ => {}
    }
    tla_to_alpha2(tla)
        .and_then(alpha2_to_emoji)
        .unwrap_or_else(|| "🏳".to_string())
}

/// Converts an ISO 3166-1 alpha-2 code to a regional indicator flag emoji.
fn alpha2_to_emoji(alpha2: &str) -> Option<String> {
    let b = alpha2.as_bytes();
    if b.len() != 2 || !b[0].is_ascii_uppercase() || !b[1].is_ascii_uppercase() {
        return None;
    }
    let c1 = char::from_u32(0x1F1E6 + (b[0] - b'A') as u32)?;
    let c2 = char::from_u32(0x1F1E6 + (b[1] - b'A') as u32)?;
    Some(format!("{c1}{c2}"))
}

/// Maps football TLAs (as used by football-data.org / UEFA / FIFA) to ISO 3166-1 alpha-2 codes.
///
/// Many TLAs differ from the ISO alpha-3 standard (e.g. "GER" instead of "DEU"),
/// so they are listed explicitly rather than derived algorithmically.
#[rustfmt::skip]
fn tla_to_alpha2(tla: &str) -> Option<&'static str> {
    match tla {
        // ── Europe ──────────────────────────────────────────────
        "ALB" => Some("AL"), // Albania
        "AND" => Some("AD"), // Andorra
        "ARM" => Some("AM"), // Armenia
        "AUT" => Some("AT"), // Austria
        "AZE" => Some("AZ"), // Azerbaijan
        "BEL" => Some("BE"), // Belgium
        "BIH" => Some("BA"), // Bosnia and Herzegovina
        "BLR" => Some("BY"), // Belarus
        "BUL" => Some("BG"), // Bulgaria (ISO: BGR)
        "CRO" => Some("HR"), // Croatia (ISO: HRV)
        "CYP" => Some("CY"), // Cyprus
        "CZE" => Some("CZ"), // Czech Republic
        "DEN" => Some("DK"), // Denmark (ISO: DNK)
        "ESP" => Some("ES"), // Spain
        "EST" => Some("EE"), // Estonia
        "FIN" => Some("FI"), // Finland
        "FRA" => Some("FR"), // France
        "GEO" => Some("GE"), // Georgia
        "GER" => Some("DE"), // Germany (ISO: DEU)
        "GIB" => Some("GI"), // Gibraltar
        "GRE" => Some("GR"), // Greece (ISO: GRC)
        "HUN" => Some("HU"), // Hungary
        "IRL" => Some("IE"), // Republic of Ireland (ISO: IRL)
        "ISL" => Some("IS"), // Iceland (ISO: ISL)
        "ISR" => Some("IL"), // Israel
        "ITA" => Some("IT"), // Italy
        "KOS" => Some("XK"), // Kosovo (unofficial alpha-2: XK)
        "LAT" => Some("LV"), // Latvia (ISO: LVA)
        "LIE" => Some("LI"), // Liechtenstein
        "LTU" => Some("LT"), // Lithuania
        "LUX" => Some("LU"), // Luxembourg
        "MDA" => Some("MD"), // Moldova
        "MKD" => Some("MK"), // North Macedonia
        "MLT" => Some("MT"), // Malta
        "MNE" => Some("ME"), // Montenegro
        "NED" => Some("NL"), // Netherlands (ISO: NLD)
        "NOR" => Some("NO"), // Norway
        "POL" => Some("PL"), // Poland
        "POR" => Some("PT"), // Portugal (ISO: PRT)
        "ROU" => Some("RO"), // Romania
        "RUS" => Some("RU"), // Russia
        "SMR" => Some("SM"), // San Marino
        "SRB" => Some("RS"), // Serbia
        "SUI" => Some("CH"), // Switzerland (ISO: CHE)
        "SVK" => Some("SK"), // Slovakia
        "SVN" => Some("SI"), // Slovenia
        "SWE" => Some("SE"), // Sweden
        "TUR" => Some("TR"), // Turkey / Türkiye
        "UKR" => Some("UA"), // Ukraine

        // ── Americas ────────────────────────────────────────────
        "ARG" => Some("AR"), // Argentina
        "BOL" => Some("BO"), // Bolivia
        "BRA" => Some("BR"), // Brazil
        "CAN" => Some("CA"), // Canada
        "CHI" => Some("CL"), // Chile (ISO: CHL)
        "COL" => Some("CO"), // Colombia
        "CRC" => Some("CR"), // Costa Rica (ISO: CRI)
        "ECU" => Some("EC"), // Ecuador
        "HON" => Some("HN"), // Honduras (ISO: HND)
        "JAM" => Some("JM"), // Jamaica
        "MEX" => Some("MX"), // Mexico
        "PAN" => Some("PA"), // Panama
        "PAR" => Some("PY"), // Paraguay (ISO: PRY)
        "PER" => Some("PE"), // Peru
        "TRI" => Some("TT"), // Trinidad and Tobago (ISO: TTO)
        "URU" => Some("UY"), // Uruguay (ISO: URY)
        "USA" => Some("US"), // United States

        // ── Africa ──────────────────────────────────────────────
        "ALG" => Some("DZ"), // Algeria (ISO: DZA)
        "ANG" => Some("AO"), // Angola (ISO: AGO)
        "CIV" => Some("CI"), // Côte d'Ivoire
        "CMR" => Some("CM"), // Cameroon
        "EGY" => Some("EG"), // Egypt
        "ETH" => Some("ET"), // Ethiopia
        "GHA" => Some("GH"), // Ghana
        "GUI" => Some("GN"), // Guinea (ISO: GIN)
        "KEN" => Some("KE"), // Kenya
        "MAR" => Some("MA"), // Morocco (ISO: MAR)
        "MLI" => Some("ML"), // Mali
        "MOZ" => Some("MZ"), // Mozambique
        "NGA" => Some("NG"), // Nigeria (ISO: NGA)
        "SEN" => Some("SN"), // Senegal
        "TUN" => Some("TN"), // Tunisia
        "ZAF" => Some("ZA"), // South Africa (sometimes "RSA")
        "RSA" => Some("ZA"), // South Africa (alternate TLA)
        "ZIM" => Some("ZW"), // Zimbabwe

        // ── Asia / Oceania ───────────────────────────────────────
        "AUS" => Some("AU"), // Australia
        "CHN" => Some("CN"), // China
        "IND" => Some("IN"), // India
        "IRN" => Some("IR"), // Iran (ISO: IRN)
        "IRQ" => Some("IQ"), // Iraq
        "JPN" => Some("JP"), // Japan
        "JOR" => Some("JO"), // Jordan (ISO: JOR)
        "KOR" => Some("KR"), // South Korea (ISO: KOR)
        "KUW" => Some("KW"), // Kuwait (ISO: KWT)
        "LBN" => Some("LB"), // Lebanon
        "NZL" => Some("NZ"), // New Zealand (ISO: NZL)
        "OMA" => Some("OM"), // Oman (ISO: OMN)
        "PHI" => Some("PH"), // Philippines
        "QAT" => Some("QA"), // Qatar
        "SAU" => Some("SA"), // Saudi Arabia (ISO: SAU)
        "SYR" => Some("SY"), // Syria
        "THA" => Some("TH"), // Thailand
        "UAE" => Some("AE"), // United Arab Emirates
        "UZB" => Some("UZ"), // Uzbekistan
        "VIE" => Some("VN"), // Vietnam (ISO: VNM)
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_tlas_return_emoji() {
        assert_eq!(flag_emoji(Some("FRA")), "🇫🇷");
        assert_eq!(flag_emoji(Some("GER")), "🇩🇪");
        assert_eq!(flag_emoji(Some("NED")), "🇳🇱");
        assert_eq!(flag_emoji(Some("ESP")), "🇪🇸");
        assert_eq!(flag_emoji(Some("POR")), "🇵🇹");
        assert_eq!(flag_emoji(Some("ITA")), "🇮🇹");
    }

    #[test]
    fn uk_nations_use_tag_sequences() {
        // These should not be the GB flag 🇬🇧
        let eng = flag_emoji(Some("ENG"));
        let sco = flag_emoji(Some("SCO"));
        let wal = flag_emoji(Some("WAL"));
        assert_ne!(eng, "🇬🇧");
        assert_ne!(sco, "🇬🇧");
        assert_ne!(wal, "🇬🇧");
        assert_ne!(eng, sco);
        assert_ne!(sco, wal);
    }

    #[test]
    fn unknown_tla_falls_back_to_white_flag() {
        assert_eq!(flag_emoji(Some("XYZ")), "🏳");
        assert_eq!(flag_emoji(None), "🏳");
    }
}
