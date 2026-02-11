use std::collections::HashMap;

#[derive(Debug, PartialEq)]
struct CueSheet {
    metadata: Metadata,
    files: Vec<FileEntry>, // a cue file can reference more than one files
    cd_text_file: Option<String>,
    catalog: Option<String>,
}

#[derive(Debug, PartialEq)]
struct Track {
    number: u8, // must be between 1 and 99
    track_type: TrackType,
    pregap: Option<Index>,
    index_00: Option<Index>,
    index_01: Index,
    additional_indexes: Vec<(u8, TrackIndex)>, // other indexes from 02 to more but it's pretty rare
    flags: Vec<Flags>,
    isrc: Option<String>,
    metadata: Metadata,
    postgap: Option<Index>,
}

#[derive(Debug, PartialEq)]
struct TrackIndex {
    number: u8,
    index: Index,
}

#[derive(Debug, PartialEq)]
struct Index {
    minute: u8,
    second: u8,
    frame: u8, // there are 75 frames per second, so 1 frame is 1/75 of a second
}

#[derive(Debug, PartialEq)]
struct FileEntry {
    file_name: String,
    file_type: FileType,
    tracks: Vec<Track>,
}

#[derive(Debug, PartialEq)]
/// Other fields and commands as specified by https://www.gnu.org/software/ccd2cue/manual/html_node/CUE-sheet-format.html,
/// - `arranger`: Name(s) of the arranger(s).
/// - `composer`: Name(s) of the composer(s).
/// - `disc_id`: Disc Identification information.
/// - `genre`: Genre Identification and Genre information.
/// - `message`: Message from the content provider and/or artist.
/// - `performer`: Name(s) of the performer(s).
/// - `songwriter`: Name(s) of the songwriter(s).
/// - `title`: Title of album name or Track Titles.
/// - `other_rem`: Additional REM comments, such as `DATE` for example.
struct Metadata {
    /// Name(s) of the arranger(s).
    arranger: Option<String>,
    /// Name(s) of the composer(s).
    composer: Option<String>,
    /// Disc Identification information.
    disc_id: Option<String>,
    /// Genre Identification and Genre information.
    genre: Option<String>,
    /// Message from the content provider and/or artist.
    message: Option<String>,
    /// Name(s) of the performer(s).
    performer: Option<String>,
    /// Name(s) of the songwriter(s).
    songwriter: Option<String>,
    /// Title of album name or Track Titles.
    title: Option<String>,
    /// Additional REM comments, such as `DATE` for example.
    other_rem: Option<HashMap<String, String>>,
}

#[derive(Debug, PartialEq)]
/// Track special sub-code flags, rarely used today.
///
/// | Value | Description |
/// |------|-------|
/// | `PRE` | **PRE**-emphasis enabled (audio tracks only) |
/// | `DCP` | **D**igital **C**opy **P**ermitted |
/// | `FourCH` *(4CH)* | **4** **CH**annel audio |
/// | `SCMS` | **S**erial **C**opy **M**anagement **S**ystem |
enum Flags {
    PRE,
    DCP,
    FourCH,
    SCMS,
}

#[derive(Debug, PartialEq)]
/// Track data modes, as defined in [ccd2cue manual](https://www.gnu.org/software/ccd2cue/manual/html_node/MODE-_0028Compact-Disc-fields_0029.html#MODE-_0028Compact-Disc-fields_0029).
///
/// *Note: The modes marked with ‘\*’ are not defined in the original CUE sheet format specification.*
///
/// | Value | Description | Sector Size |
/// |------|-------|-----|
/// | `AUDIO` | Audio/Music (588 samples) | 2352 |
/// | `CDG` | Karaoke CD+G | 2448 |
/// | `MODE1/2048` | CD-ROM Mode 1 Data (cooked) | 2048 |
/// | `MODE1/2352` | CD-ROM Mode 1 Data (raw) | 2352 |
/// | `MODE2/2048` | CD-ROM XA Mode 2 Data (form 1) * | 2048 |
/// | `MODE2/2324` | CD-ROM XA Mode 2 Data (form 2) * | 2324 |
/// | `MODE2/2336` | CD-ROM XA Mode 2 Data (form mix) | 2336 |
/// | `MODE2/2352` | CD-ROM XA Mode 2 Data (raw) | 2352 |
/// | `CDI/2336` | CD-i Mode 2 Data | 2336 |
/// | `CDI/2352` | CD-i Mode 2 Data (raw) | 2352 |
enum TrackType {
    Audio,
    CDG,
    Mode1_2048,
    Mode1_2352,
    Mode2_2048,
    Mode2_2324,
    Mode2_2336,
    Mode2_2352,
    CDI_2336,
    CDI_2352,
}

#[derive(Debug, PartialEq)]
enum FileType {
    Binary,
    Motorola,
    Aiff,
    Wave,
    Mp3,
}
