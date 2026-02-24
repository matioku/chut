use std::collections::HashMap;

/// Represents a complete CUE sheet file structure.
///
/// A CUE sheet describes the layout of tracks on a CD or disc image. It contains
/// global metadata, catalog information, and references to one or more audio/data files,
/// each containing their own tracks.
///
/// # Format Reference
/// Based on the [CUE sheet format specification](https://www.gnu.org/software/ccd2cue/manual/html_node/CUE-sheet-format.html).
///
/// # Examples
/// ```
/// # use cue_parser::*;
/// let cue = CueSheet {
///     catalog: Some("1234567890123".to_string()),
///     cd_text_file: None,
///     metadata: Metadata::default(),
///     files: vec![],
/// };
#[derive(Debug, PartialEq, Clone)]
struct CueSheet {
    /// UPC/EAN catalog number (13 numeric digits).
    ///
    /// Must be a numeric value of exactly 13 digits, encoded according to
    /// UPC/EAN (Universal Product Code/European Article Number) rules.
    /// Can appear only once per CUE sheet.
    ///
    /// Specified by the `CATALOG` command.
    catalog: Option<String>,
    /// Path to an external CD-TEXT file.
    ///
    /// Must be enclosed in quotation marks if it contains spaces.
    /// CD-TEXT files contain additional metadata like track titles and artist names
    /// that can be read by CD-TEXT compatible players.
    ///
    /// Specified by the `CDTEXTFILE` command.
    cd_text_file: Option<String>,
    /// Album-level metadata (title, performer, etc.).
    ///
    /// Contains global metadata that applies to the entire disc/album.
    /// Individual tracks can override these values with their own metadata.
    metadata: Metadata,
    /// List of files referenced by this CUE sheet.
    ///
    /// A CUE sheet can reference more than one file. Each file contains one or more tracks.
    /// Each `FILE` command in the CUE sheet creates a new `FileEntry`.
    ///
    /// Most CUE sheets reference only a single file, but multi-file CUE sheets are valid
    /// according to the specification.
    files: Vec<FileEntry>,
}

/// Represents a single track within a CUE sheet.
///
/// Tracks are numbered sequentially from 1 to 99 and describe individual audio or data
/// sections within a file. Each track has a mandatory start position (`index_01`) and
/// optional additional indexes for subdivisions.
///
/// # Track Numbering
/// - Track numbers must be between 1 and 99 (inclusive)
/// - Tracks should be numbered sequentially, but gaps are technically allowed
///
/// # Index Points
/// - `INDEX 00`: Optional pregap stored in the file (hidden audio)
/// - `INDEX 01`: Mandatory start position of the track (required)
/// - `INDEX 02+`: Optional sub-indexes for track subdivisions (rare)
///
/// # Format Reference
/// See the [TRACK command specification](https://www.gnu.org/software/ccd2cue/manual/html_node/TRACK-_0028Compact-Disc-fields_0029.html).
#[derive(Debug, PartialEq, Clone)]
pub struct Track {
    /// Track number (1-99).
    ///
    /// Must be between 1 and 99 inclusive. Tracks are typically numbered sequentially
    /// starting from 1, though the specification allows gaps in numbering.
    number: u8,
    /// Data type/mode of the track.
    ///
    /// Specifies whether this track contains audio (AUDIO), CD+G karaoke data (CDG),
    /// or various CD-ROM data modes. The track type determines how sectors are interpreted
    /// and their size.
    ///
    /// Specified immediately after the `TRACK` command, e.g., `TRACK 01 AUDIO`.
    track_type: TrackType,
    /// Generated pregap (silence before the track).
    ///
    /// Specifies a duration of artificial silence to insert before this track starts.
    /// This silence is **not** stored in the audio file—it's generated during burning/playback.
    ///
    /// Specified by the `PREGAP` command with format `MM:SS:FF`.
    ///
    /// **Mutually exclusive** with `index_00`. A track can have either a `PREGAP` command
    /// or an `INDEX 00`, but not both.
    ///
    /// Must appear after a `TRACK` command but before any `INDEX` commands.
    /// Only one `PREGAP` command is allowed per track.
    pregap: Option<Index>,
    /// Pregap stored in the file (INDEX 00).
    ///
    /// Points to the start position of a pregap that **exists in the audio file**.
    /// The region between `index_00` and `index_01` is treated as a pregap (often silence,
    /// but can contain hidden audio).
    ///
    /// **Mutually exclusive** with `pregap`. A track can have either `index_00` or `pregap`,
    /// but not both.
    ///
    /// Common use case: "Hidden tracks" at the start of Track 01.
    index_00: Option<Index>,
    /// Start position of the track (INDEX 01) - **mandatory**.
    ///
    /// Specifies the absolute position in the file where this track begins.
    /// This is the position a CD player will jump to when selecting this track.
    ///
    /// Every track **must** have an `INDEX 01`. This is required by the CUE sheet specification.
    ///
    /// Format: `INDEX 01 MM:SS:FF`
    index_01: Index,
    /// Additional sub-indexes (INDEX 02, 03, ...).
    ///
    /// Optional subdivision points within a track. These are rarely used but can mark
    /// movements in classical music, sections in DJ sets, or chapters in long recordings.
    ///
    /// Index numbers must be sequential and greater than 1 (2, 3, 4, ...).
    ///
    /// Most tracks have zero additional indexes.
    additional_indexes: Vec<TrackIndex>,
    /// Track sub-code flags.
    ///
    /// Specifies special CD sub-code flags for this track. Rarely used in modern CUE sheets.
    ///
    /// Possible flags:
    /// - `PRE`: Pre-emphasis enabled (audio tracks only)
    /// - `DCP`: Digital Copy Permitted
    /// - `4CH`: Four-channel audio
    /// - `SCMS`: Serial Copy Management System
    ///
    /// Specified by the `FLAGS` command, e.g., `FLAGS DCP PRE`.
    flags: Vec<Flags>,
    /// International Standard Recording Code for this track.
    ///
    /// A unique 12-character alphanumeric code that identifies a specific recording.
    /// Typically used for commercial CDs to enable royalty tracking and identification.
    ///
    /// Format: `CC-XXX-YY-NNNNN` (though often stored without hyphens)
    ///
    /// Specified by the `ISRC` command.
    ///
    /// # Example
    /// ```text
    /// ISRC USUM71234567
    /// ```
    isrc: Option<String>,
    /// Track-specific metadata (title, performer, etc.).
    ///
    /// Contains metadata that applies to this specific track. Values here override
    /// the global metadata from the parent `CueSheet`.
    metadata: Metadata,
    /// Generated silence after the track (POSTGAP).
    ///
    /// Specifies a duration of silence to append after this track ends.
    /// Like `pregap`, this silence is generated and **not stored in the file**.
    ///
    /// Postgaps are considered not to be stored in the file specified by the `FILE` command.
    /// They are generated during burning/playback.
    ///
    /// Specified by the `POSTGAP` command with format `MM:SS:FF`.
    ///
    /// Very rarely used in practice.
    postgap: Option<Index>,
}

/// An index point within a track (INDEX command).
///
/// Represents a specific `INDEX` command within a track. Indexes mark positions
/// in the audio file using the format `MM:SS:FF` (minutes:seconds:frames).
///
/// - `INDEX 00`: Pregap start (optional)
/// - `INDEX 01`: Track start (mandatory)
/// - `INDEX 02+`: Sub-divisions (optional, rare)
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TrackIndex {
    /// Index number (0, 1, 2, 3, ...).
    ///
    /// - `0`: Pregap start
    /// - `1`: Track start (mandatory)
    /// - `2+`: Additional sub-indexes
    number: u8,
    /// Position in the file where this index points.
    ///
    /// Absolute time position using the format `MM:SS:FF`.
    index: Index,
}

/// A time position in the format MM:SS:FF (minutes:seconds:frames).
///
/// Represents an absolute position in an audio file. The format is:
/// - **Minutes**: 0-99 (no upper limit in practice)
/// - **Seconds**: 0-59
/// - **Frames**: 0-74 (there are exactly 75 frames per second)
///
/// # Frame Rate
/// CD audio uses 75 frames per second, meaning:
/// - 1 frame = 1/75 second ≈ 13.33 milliseconds
/// - 1 second = 75 frames
///
/// # Examples
/// ```
/// # use cue_parser::Index;
/// // 3 minutes, 30 seconds, 0 frames = 3:30.000
/// let idx = Index { minute: 3, second: 30, frame: 0 };
///
/// // 0 minutes, 2 seconds, 37 frames = 0:02.493
/// let idx2 = Index { minute: 0, second: 2, frame: 37 };
/// ```
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Index {
    /// Minutes component (0-255, typically 0-99).
    pub minute: u8,
    /// Seconds component (must be 0-59).
    pub second: u8,
    /// Frames component (must be 0-74).
    ///
    /// There are exactly 75 frames per second in CD audio,
    /// so 1 frame = 1/75 second ≈ 13.33 milliseconds.
    pub frame: u8,
}

/// Represents a file referenced in a CUE sheet (FILE command).
///
/// A CUE sheet can reference one or more files. Each file contains one or more tracks.
/// The file type specifies the format of the audio/data file.
///
/// # Format
/// ```text
/// FILE "filename.wav" WAVE
///   TRACK 01 AUDIO
///     INDEX 01 00:00:00
/// ```
///
/// Most CUE sheets reference a single file, but multi-file CUE sheets are valid.
#[derive(Debug, PartialEq, Clone)]
pub struct FileEntry {
    /// Filename or path to the audio/data file.
    ///
    /// Can be:
    /// - A simple filename: `"album.flac"`
    /// - A relative path: `"audio/disc1.bin"`
    /// - An absolute path: `"/home/user/music/album.wav"`
    ///
    /// Should be enclosed in quotes if it contains spaces (enforced during parsing).
    pub file_name: String,
    /// Format/type of the file.
    ///
    /// Specifies how the file data should be interpreted. Common types:
    /// - `WAVE`: PCM audio in WAV format
    /// - `BINARY`: Raw binary data (for CD images)
    /// - `MP3`: MPEG Layer 3 audio
    /// - `AIFF`: Audio Interchange File Format
    pub file_type: FileType,
    /// Tracks contained within this file.
    ///
    /// All `TRACK` commands following a `FILE` command belong to that file,
    /// until the next `FILE` command is encountered.
    ///
    /// A file must contain at least one track.
    pub tracks: Vec<Track>,
}

/// Metadata fields for albums and tracks.
///
/// Contains CD-TEXT and REM comment metadata as specified by the
/// [CUE sheet format specification](https://www.gnu.org/software/ccd2cue/manual/html_node/CUE-sheet-format.html).
///
/// All string fields should be limited to **80 characters** per the specification,
/// though this is not strictly enforced by all software.
///
/// # Usage
/// - At the **album level**: Metadata applies to the entire disc
/// - At the **track level**: Metadata overrides album-level values for that track
///
/// # REM Comments
/// The `other_rem` field captures custom `REM` comments like `DATE`, `COMMENT`, etc.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Metadata {
    /// Name(s) of the arranger(s).
    ///
    /// Specified by: `ARRANGER "name"`
    pub arranger: Option<String>,
    /// Name(s) of the composer(s).
    ///
    /// Specified by: `COMPOSER "name"`
    pub composer: Option<String>,
    /// Disc identification information.
    ///
    /// Often used to store disc IDs from online databases like FreeDB or MusicBrainz.
    ///
    /// Specified by: `REM DISCID <id>`
    pub disc_id: Option<String>,
    /// Genre identification and information.
    ///
    /// Can be a standard genre name (e.g., "Rock", "Jazz") or a numeric genre code.
    ///
    /// Specified by: `REM GENRE "genre name"`
    pub genre: Option<String>,
    /// Message from the content provider and/or artist.
    ///
    /// Freeform text message.
    ///
    /// Specified by: `MESSAGE "message text"`
    pub message: Option<String>,
    /// Name(s) of the performer(s) or artist(s).
    ///
    /// At the album level: Album artist
    /// At the track level: Track artist
    ///
    /// Specified by: `PERFORMER "name"`
    pub performer: Option<String>,
    /// Name(s) of the songwriter(s).
    ///
    /// Specified by: `SONGWRITER "name"`
    pub songwriter: Option<String>,
    /// Title of the album or track.
    ///
    /// At the album level: Album title
    /// At the track level: Track title
    ///
    /// Specified by: `TITLE "title text"`
    pub title: Option<String>,
    /// Additional REM comments not covered by standard fields.
    ///
    /// Stores custom `REM` comments as key-value pairs.
    ///
    /// Common examples:
    /// - `DATE`: Release year or date (`REM DATE 2023`)
    /// - `COMMENT`: Arbitrary comments (`REM COMMENT "Remastered edition"`)
    /// - `DISCNUMBER`: Disc number in multi-disc sets
    ///
    /// # Example
    /// ```text
    /// REM DATE 2023
    /// REM COMMENT "Remastered"
    /// ```
    /// Results in:
    /// ```
    /// # use std::collections::HashMap;
    /// let mut map = HashMap::new();
    /// map.insert("DATE".to_string(), "2023".to_string());
    /// map.insert("COMMENT".to_string(), "Remastered".to_string());
    /// ```
    pub other_rem: HashMap<String, String>,
}

/// Track sub-code flags (FLAGS command).
///
/// Specifies special sub-code flags that can be set on CD tracks.
/// These flags are rarely used in modern CUE sheets but are part of the
/// [Red Book CD-DA standard](https://en.wikipedia.org/wiki/Compact_Disc_Digital_Audio).
///
/// # Usage
/// ```text
/// TRACK 01 AUDIO
///   FLAGS DCP PRE
/// ```
///
/// Multiple flags can be combined.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Flags {
    /// **PRE**-emphasis enabled (audio tracks only).
    ///
    /// A pre-emphasis filter was applied during recording. Players should apply
    /// de-emphasis during playback to restore the original frequency response.
    ///
    /// Rarely used today; was common in early digital audio to reduce noise.
    Pre,
    /// **D**igital **C**opy **P**ermitted.
    ///
    /// Indicates that digital copying of this track is permitted.
    /// Part of the Serial Copy Management System (SCMS).
    Dcp,
    /// **4** **CH**annel audio.
    ///
    /// Indicates that this track contains 4-channel (quadraphonic) audio
    /// instead of standard 2-channel stereo.
    ///
    /// Very rarely used.
    FourCh,
    /// **S**erial **C**opy **M**anagement **S**ystem enabled.
    ///
    /// Part of the SCMS copy protection scheme. Limits the number of
    /// serial digital copies that can be made.
    Scms,
}

/// Track data type/mode (TRACK command second parameter).
///
/// Specifies the format and sector size of track data. Audio tracks use `AUDIO`,
/// while data tracks use various MODE specifications depending on the CD format.
///
/// As defined in the [ccd2cue manual](https://www.gnu.org/software/ccd2cue/manual/html_node/MODE-_0028Compact-Disc-fields_0029.html#MODE-_0028Compact-Disc-fields_0029).
///
/// *Note: Modes marked with '\*' are extensions not in the original CDRWIN specification.*
///
/// # Examples
/// ```text
/// TRACK 01 AUDIO         ; Audio track
/// TRACK 02 MODE1/2048    ; CD-ROM data track
/// TRACK 03 CDG           ; CD+Graphics (karaoke)
/// ```
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TrackType {
    /// Audio/Music (2352 bytes per sector).
    ///
    /// Standard Red Book CD-DA audio: 44.1 kHz, 16-bit, stereo, 588 samples per sector.
    Audio,
    /// Karaoke CD+Graphics (2448 bytes per sector).
    ///
    /// CD+G stores graphics alongside audio for karaoke displays.
    /// 2352 bytes audio + 96 bytes graphics per sector.
    Cdg,
    /// CD-ROM Mode 1 Data, cooked (2048 bytes per sector).
    ///
    /// Standard CD-ROM data mode. Only user data, no error correction or headers.
    /// Most common data track type.
    Mode1_2048,
    /// CD-ROM Mode 1 Data, raw (2352 bytes per sector).
    ///
    /// Full raw sector including sync, headers, user data, and error correction (ECC/EDC).
    Mode1_2352,
    /// CD-ROM XA Mode 2 Data, form 1 (2048 bytes per sector). *
    ///
    /// CD-ROM XA (eXtended Architecture) Mode 2, Form 1.
    /// User data only, similar to Mode1/2048.
    ///
    /// *Extension: Not in original CDRWIN specification.*
    Mode2_2048,
    /// CD-ROM XA Mode 2 Data, form 2 (2324 bytes per sector). *
    ///
    /// CD-ROM XA Mode 2, Form 2.
    /// More user data, less error correction (used for video/audio streams).
    ///
    /// *Extension: Not in original CDRWIN specification.*
    Mode2_2324,
    /// CD-ROM XA Mode 2 Data, form mix (2336 bytes per sector).
    ///
    /// CD-ROM XA Mode 2 with mixed forms (Form 1 and Form 2 sectors).
    /// Contains sub-header and user data, no sync/header.
    Mode2_2336,
    /// CD-ROM XA Mode 2 Data, raw (2352 bytes per sector).
    ///
    /// Full raw Mode 2 XA sector.
    Mode2_2352,
    /// CD-i Mode 2 Data (2336 bytes per sector).
    ///
    /// Philips CD-i (Compact Disc Interactive) format.
    Cdi2336,
    /// CD-i Mode 2 Data, raw (2352 bytes per sector).
    ///
    /// Full raw CD-i sector.
    Cdi2352,
}

/// File format type (FILE command second parameter).
///
/// Specifies the format of the file referenced by a `FILE` command.
/// The file type tells burning/playback software how to interpret the file data.
///
/// # Examples
/// ```text
/// FILE "album.wav" WAVE
/// FILE "disc.bin" BINARY
/// FILE "track.mp3" MP3
/// ```
///
/// # Common Types
/// - **WAVE**: Most common for audio CUE sheets (lossless PCM audio)
/// - **BINARY**: Used for CD images (BIN/CUE pairs)
/// - **MP3**: Supported by some modern software for convenience
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FileType {
    /// Intel binary file (raw data, little-endian).
    ///
    /// Most common for CD/DVD images. Used with BIN/CUE pairs.
    /// Contains raw sector data as it appears on the disc.
    Binary,
    /// Motorola binary file (raw data, big-endian).
    ///
    /// Same as `Binary` but with big-endian byte order.
    /// Rarely used in practice.
    Motorola,
    /// Audio Interchange File Format (Apple/SGI).
    ///
    /// Uncompressed audio format, primarily used on Apple and Unix systems.
    Aiff,
    /// Waveform Audio File Format (Microsoft/IBM).
    ///
    /// Most common audio format for CUE sheets. Contains uncompressed PCM audio.
    /// Typically 44.1 kHz, 16-bit, stereo for CD audio.
    Wave,
    /// MPEG-1 Audio Layer 3 (compressed audio).
    ///
    /// Lossy compressed audio format. Supported by many modern CD burning tools
    /// and players, though not part of the original CDRWIN specification.
    Mp3,
}
