// Handle different storage classes
#![forbid(unsafe_code)]
use std::fmt;

#[derive(Debug, PartialEq)]
enum StorageClass {
    DeepArchive,
    Glacier,
    IntelligentTiering,
    OneZoneIA,
    ReducedRedundancy,
    Standard,
    StandardIA,

    // Catch possible future storage types
    Unknown(String),
}

impl fmt::Display for StorageClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::DeepArchive        => "DeepArchive",
            Self::Glacier            => "Glacier",
            Self::IntelligentTiering => "IntelligentTiering",
            Self::OneZoneIA          => "OneZoneIA",
            Self::ReducedRedundancy  => "ReducedRedundancy",
            Self::Standard           => "Standard",
            Self::StandardIA         => "StandardIA",
            Self::Unknown(unknown)   => unknown,
        };

        write!(f, "{}", s)
    }
}

impl From<&str> for StorageClass {
    fn from(s: &str) -> Self {
        match s {
            // Strings from CloudWatch
            "DeepArchiveStorage"          => Self::DeepArchive,
            "DeepArchiveObjectOverhead"   => Self::DeepArchive,
            "DeepArchiveS3ObjectOverhead" => Self::DeepArchive,
            "DeepArchiveStagingStorage"   => Self::DeepArchive,
            "GlacierObjectOverhead"       => Self::Glacier,
            "GlacierStorage"              => Self::Glacier,
            "GlacierStagingStorage"       => Self::Glacier,
            "GlacierS3ObjectOverhead"     => Self::Glacier,
            "IntelligentTieringStorage"   => Self::IntelligentTiering,
            "OneZoneIASizeOverhead"       => Self::OneZoneIA,
            "OneZoneIAStorage"            => Self::OneZoneIA,
            "ReducedRedundancyStorage"    => Self::ReducedRedundancy,
            "StandardIAObjectOverhead"    => Self::StandardIA,
            "StandardIASizeOverhead"      => Self::StandardIA,
            "StandardIAStorage"           => Self::StandardIA,
            "StandardStorage"             => Self::Standard,

            // Strings from S3
            "DEEP_ARCHIVE"        => Self::DeepArchive,
            "GLACIER"             => Self::Glacier,
            "INTELLIGENT_TIERING" => Self::IntelligentTiering,
            "ONEZONE_IA"          => Self::OneZoneIA,
            "REDUCED_REDUNDANCY"  => Self::ReducedRedundancy,
            "STANDARD"            => Self::Standard,
            "STANDARD_IA"         => Self::StandardIA,

            // Possible future types
            unknown => Self::Unknown(unknown.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_from_str() {
        let tests = vec![
            // CloudWatch
            ("DeepArchiveStorage",          StorageClass::DeepArchive),
            ("DeepArchiveObjectOverhead",   StorageClass::DeepArchive),
            ("DeepArchiveS3ObjectOverhead", StorageClass::DeepArchive),
            ("DeepArchiveStagingStorage",   StorageClass::DeepArchive),
            ("GlacierObjectOverhead",       StorageClass::Glacier),
            ("GlacierStorage",              StorageClass::Glacier),
            ("GlacierStagingStorage",       StorageClass::Glacier),
            ("GlacierS3ObjectOverhead",     StorageClass::Glacier),
            ("IntelligentTieringStorage",   StorageClass::IntelligentTiering),
            ("OneZoneIASizeOverhead",       StorageClass::OneZoneIA),
            ("OneZoneIAStorage",            StorageClass::OneZoneIA),
            ("ReducedRedundancyStorage",    StorageClass::ReducedRedundancy),
            ("StandardIAObjectOverhead",    StorageClass::StandardIA),
            ("StandardIASizeOverhead",      StorageClass::StandardIA),
            ("StandardIAStorage",           StorageClass::StandardIA),
            ("StandardStorage",             StorageClass::Standard),

            // S3
            ("DEEP_ARCHIVE",        StorageClass::DeepArchive),
            ("GLACIER",             StorageClass::Glacier),
            ("INTELLIGENT_TIERING", StorageClass::IntelligentTiering),
            ("ONEZONE_IA",          StorageClass::OneZoneIA),
            ("REDUCED_REDUNDANCY",  StorageClass::ReducedRedundancy),
            ("STANDARD",            StorageClass::Standard),
            ("STANDARD_IA",         StorageClass::StandardIA),

            // Unknown
            ("Wat", StorageClass::Unknown("Wat".into())),
        ];

        for test in tests {
            let input    = test.0;
            let expected = test.1;

            let ret = StorageClass::from(input);

            assert_eq!(ret, expected);
        }
    }

    #[test]
    fn test_into_string() {
        let tests = vec![
            (StorageClass::DeepArchive,            "DeepArchive"),
            (StorageClass::Glacier,                "Glacier"),
            (StorageClass::IntelligentTiering,     "IntelligentTiering"),
            (StorageClass::OneZoneIA,              "OneZoneIA"),
            (StorageClass::ReducedRedundancy,      "ReducedRedundancy"),
            (StorageClass::Standard,               "Standard"),
            (StorageClass::StandardIA,             "StandardIA"),
            (StorageClass::Unknown("test".into()), "test"),
        ];

        for test in tests {
            let storage_class = test.0;
            let expected      = test.1;

            let ret = storage_class.to_string();

            assert_eq!(ret, expected);
        }
    }
}
