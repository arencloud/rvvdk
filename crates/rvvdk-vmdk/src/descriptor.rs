use thiserror::Error;

pub const SECTOR_BYTES: u64 = 512;

/// Resource ceilings, checked before growing parser-owned collections.
/// The input belongs to the caller; an eventual reader must bound acquisition too.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub descriptor_bytes: usize,
    pub line_bytes: usize,
    pub extents: usize,
    pub metadata_entries: usize,
    pub filename_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            descriptor_bytes: 1024 * 1024,
            line_bytes: 8192,
            extents: 1024,
            metadata_entries: 128,
            filename_bytes: 4096,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreateType {
    MonolithicFlat,
    TwoGbMaxExtentFlat,
    Custom,
    MonolithicSparse,
    TwoGbMaxExtentSparse,
}

/// Descriptor permissions; even RW does not authorize writes through this crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    ReadWrite,
    ReadOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtentBacking<'a> {
    /// A lexical reference, not a validated or opened filesystem path.
    Flat {
        file_name: &'a str,
        offset_bytes: u64,
    },
    Zero,
    Sparse {
        file_name: &'a str,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Extent<'a> {
    access: Access,
    logical_offset: u64,
    size_bytes: u64,
    backing: ExtentBacking<'a>,
}

impl<'a> Extent<'a> {
    pub fn access(&self) -> Access {
        self.access
    }
    pub fn logical_offset(&self) -> u64 {
        self.logical_offset
    }
    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }
    pub fn backing(&self) -> ExtentBacking<'a> {
        self.backing
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Metadata<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

/// Validated layout metadata borrowing the input. This is not a VirtualDisk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Descriptor<'a> {
    cid: u32,
    create_type: CreateType,
    size_bytes: u64,
    extents: Vec<Extent<'a>>,
    metadata: Vec<Metadata<'a>>,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ErrorKind {
    #[error("resource limit exceeded: {0}")]
    Limit(&'static str),
    #[error("invalid UTF-8 or unsupported control character")]
    Encoding,
    #[error("invalid syntax: {0}")]
    Syntax(&'static str),
    #[error("duplicate field")]
    Duplicate,
    #[error("missing required field: {0}")]
    Missing(&'static str),
    #[error("unsupported feature: {0}")]
    Unsupported(&'static str),
    #[error("invalid unsigned integer or CID")]
    Number,
    #[error("sector/byte arithmetic overflow")]
    Overflow,
    #[error("invalid extent layout: {0}")]
    Layout(&'static str),
}

/// One-based source line; zero means an input-wide or final validation error.
/// Messages never echo untrusted filenames or descriptor content.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[error("VMDK descriptor line {line}: {kind}")]
pub struct DescriptorError {
    pub line: usize,
    pub kind: ErrorKind,
}

type Result<T> = std::result::Result<T, DescriptorError>;
fn error(line: usize, kind: ErrorKind) -> DescriptorError {
    DescriptorError { line, kind }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token<'a> {
    Bare(&'a str),
    Quoted(&'a str),
    Equal,
}

// A fixed token buffer bounds malformed per-line work without token allocations.
fn tokens(line: &str, number: usize) -> Result<([Token<'_>; 6], usize)> {
    let mut out = [Token::Equal; 6];
    let mut count = 0;
    let mut rest = line.trim_matches([' ', '\t']);
    while !rest.is_empty() && !rest.starts_with('#') {
        if count == out.len() {
            return Err(error(number, ErrorKind::Syntax("too many tokens")));
        }
        let (token, consumed) = if let Some(quoted) = rest.strip_prefix('"') {
            let end = quoted
                .find('"')
                .ok_or(error(number, ErrorKind::Syntax("unterminated quote")))?;
            let value = &quoted[..end];
            if value.chars().any(char::is_control) {
                return Err(error(number, ErrorKind::Encoding));
            }
            if value.contains('\\') {
                return Err(error(
                    number,
                    ErrorKind::Unsupported("quoted backslash/escape"),
                ));
            }
            (Token::Quoted(value), end + 2)
        } else if rest.starts_with('=') {
            (Token::Equal, 1)
        } else {
            let end = rest.find([' ', '\t', '=', '"', '#']).unwrap_or(rest.len());
            (Token::Bare(&rest[..end]), end)
        };
        rest = &rest[consumed..];
        if matches!(token, Token::Bare(_)) && rest.starts_with('"') {
            return Err(error(number, ErrorKind::Syntax("missing token separator")));
        }
        if matches!(token, Token::Quoted(_))
            && !rest.is_empty()
            && !rest.starts_with([' ', '\t', '#'])
        {
            return Err(error(number, ErrorKind::Syntax("text after quote")));
        }
        out[count] = token;
        count += 1;
        rest = rest.trim_start_matches([' ', '\t']);
    }
    Ok((out, count))
}

fn number(value: &str, line: usize) -> Result<u64> {
    if value.is_empty() || !value.bytes().all(|c| c.is_ascii_digit()) {
        return Err(error(line, ErrorKind::Number));
    }
    value.parse().map_err(|_| error(line, ErrorKind::Number))
}
fn cid<const SPARSE: bool>(value: &str, line: usize) -> Result<u32> {
    if value.is_empty()
        || value.len() > 8
        || (!SPARSE && value.len() != 8)
        || !value.bytes().all(|c| c.is_ascii_hexdigit())
    {
        return Err(error(line, ErrorKind::Number));
    }
    u32::from_str_radix(value, 16).map_err(|_| error(line, ErrorKind::Number))
}
fn bytes(value: &str, line: usize) -> Result<u64> {
    number(value, line)?
        .checked_mul(SECTOR_BYTES)
        .ok_or(error(line, ErrorKind::Overflow))
}
fn set<T>(slot: &mut Option<T>, value: T, line: usize) -> Result<()> {
    if slot.is_some() {
        return Err(error(line, ErrorKind::Duplicate));
    }
    *slot = Some(value);
    Ok(())
}
fn eq(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}
const DDB_KEYS: &[&str] = &[
    "ddb.adapterType",
    "ddb.geometry.cylinders",
    "ddb.geometry.heads",
    "ddb.geometry.sectors",
    "ddb.virtualHWVersion",
    "ddb.toolsVersion",
    "ddb.uuid",
    "ddb.longContentID",
];

impl<'a> Descriptor<'a> {
    pub fn cid(&self) -> u32 {
        self.cid
    }
    pub fn create_type(&self) -> CreateType {
        self.create_type
    }
    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }
    pub fn extents(&self) -> &[Extent<'a>] {
        &self.extents
    }
    /// Informational values only; never used to infer capacity or sector size.
    pub fn metadata(&self) -> &[Metadata<'a>] {
        &self.metadata
    }

    pub fn parse(input: &'a [u8]) -> Result<Self> {
        Self::parse_with_limits(input, Limits::default())
    }

    pub fn parse_with_limits(input: &'a [u8], limits: Limits) -> Result<Self> {
        Self::parse_subset::<false, false>(input, limits).map(|(d, _)| d)
    }

    fn parse_subset<const SPARSE: bool, const CHAIN: bool>(
        input: &'a [u8],
        limits: Limits,
    ) -> Result<(Self, Option<ParentReference<'a>>)> {
        if input.len() > limits.descriptor_bytes {
            return Err(error(0, ErrorKind::Limit("descriptor bytes")));
        }
        let text = std::str::from_utf8(input).map_err(|_| error(0, ErrorKind::Encoding))?;
        let mut version = None;
        let mut content_id = None;
        let mut parent = None;
        let mut parent_hint = None;
        let mut create_type = None;
        let mut encoding = None;
        let mut extents = Vec::new();
        let mut metadata: Vec<Metadata<'a>> = Vec::new();
        let mut size_bytes = 0_u64;
        let mut section = 0; // header -> extents -> DDB, never backwards
        for (i, raw) in text.split('\n').enumerate() {
            let line = i + 1;
            if raw.len() > limits.line_bytes {
                return Err(error(line, ErrorKind::Limit("line bytes")));
            }
            let raw = raw.strip_suffix('\r').unwrap_or(raw);
            if raw.chars().any(|c| c.is_control() && c != '\t') {
                return Err(error(line, ErrorKind::Encoding));
            }
            let (buffer, count) = tokens(raw, line)?;
            let t = &buffer[..count];
            if t.is_empty() {
                continue;
            }
            if let [Token::Bare(key), Token::Equal, value] = t {
                if key.get(..4).is_some_and(|prefix| eq(prefix, "ddb.")) {
                    if section == 0 {
                        return Err(error(line, ErrorKind::Syntax("DDB before extents")));
                    }
                    section = 2;
                    if !DDB_KEYS.iter().any(|allowed| eq(key, allowed)) {
                        return Err(error(line, ErrorKind::Unsupported("DDB key")));
                    }
                    let Token::Quoted(value) = value else {
                        return Err(error(line, ErrorKind::Syntax("DDB value must be quoted")));
                    };
                    if metadata.iter().any(|entry| eq(entry.key, key)) {
                        return Err(error(line, ErrorKind::Duplicate));
                    }
                    if metadata.len() >= limits.metadata_entries {
                        return Err(error(line, ErrorKind::Limit("metadata entries")));
                    }
                    metadata.push(Metadata { key, value });
                    continue;
                }
                if section != 0 {
                    return Err(error(line, ErrorKind::Syntax("header after extents")));
                }
                match (key, value) {
                    (k, Token::Bare(v)) if eq(k, "version") => {
                        if number(v, line)? != 1 {
                            return Err(error(line, ErrorKind::Unsupported("descriptor version")));
                        }
                        set(&mut version, (), line)?;
                    }
                    (k, Token::Bare(v)) if eq(k, "CID") => {
                        set(&mut content_id, cid::<SPARSE>(v, line)?, line)?
                    }
                    (k, Token::Bare(v)) if eq(k, "parentCID") => {
                        let value = cid::<SPARSE>(v, line)?;
                        if !CHAIN && value != u32::MAX {
                            return Err(error(line, ErrorKind::Unsupported("parent chain")));
                        }
                        set(&mut parent, value, line)?;
                    }
                    (k, Token::Quoted(v)) if eq(k, "createType") => {
                        let kind = if SPARSE && eq(v, "monolithicSparse") {
                            CreateType::MonolithicSparse
                        } else if SPARSE
                            && (eq(v, "twoGbMaxExtentSparse") || eq(v, "2GbMaxExtentSparse"))
                        {
                            CreateType::TwoGbMaxExtentSparse
                        } else if !SPARSE && eq(v, "monolithicFlat") {
                            CreateType::MonolithicFlat
                        } else if !SPARSE
                            && (eq(v, "twoGbMaxExtentFlat") || eq(v, "2GbMaxExtentFlat"))
                        {
                            CreateType::TwoGbMaxExtentFlat
                        } else if !SPARSE && eq(v, "custom") {
                            CreateType::Custom
                        } else {
                            return Err(error(line, ErrorKind::Unsupported("create type")));
                        };
                        set(&mut create_type, kind, line)?;
                    }
                    (k, Token::Quoted(v)) if eq(k, "encoding") => {
                        if !eq(v, "UTF-8") {
                            return Err(error(line, ErrorKind::Unsupported("encoding")));
                        }
                        set(&mut encoding, (), line)?;
                    }
                    (k, Token::Quoted(v)) if CHAIN && eq(k, "parentFileNameHint") => {
                        if v.is_empty() {
                            return Err(error(line, ErrorKind::Layout("empty parent hint")));
                        }
                        if v.len() > limits.filename_bytes {
                            return Err(error(line, ErrorKind::Limit("parent hint bytes")));
                        }
                        set(&mut parent_hint, *v, line)?;
                    }
                    (k, _) if eq(k, "parentFileNameHint") => {
                        return Err(error(line, ErrorKind::Unsupported("parent chain")));
                    }
                    (k, _)
                        if ["version", "CID", "parentCID", "createType", "encoding"]
                            .iter()
                            .any(|v| eq(k, v)) =>
                    {
                        return Err(error(line, ErrorKind::Syntax("header value quoting")));
                    }
                    _ => return Err(error(line, ErrorKind::Unsupported("header key"))),
                }
                continue;
            }
            if section == 2 {
                return Err(error(line, ErrorKind::Syntax("extent after DDB")));
            }
            section = 1;
            if extents.len() >= limits.extents {
                return Err(error(line, ErrorKind::Limit("extents")));
            }
            let [
                Token::Bare(access),
                Token::Bare(sectors),
                Token::Bare(kind),
                tail @ ..,
            ] = t
            else {
                return Err(error(line, ErrorKind::Syntax("extent fields")));
            };
            let access = if eq(access, "RW") {
                Access::ReadWrite
            } else if eq(access, "RDONLY") {
                Access::ReadOnly
            } else {
                return Err(error(line, ErrorKind::Unsupported("extent access")));
            };
            let length = bytes(sectors, line)?;
            if length == 0 {
                return Err(error(line, ErrorKind::Layout("empty extent")));
            }
            let backing = if SPARSE && eq(kind, "SPARSE") {
                let [Token::Quoted(name)] = tail else {
                    return Err(error(line, ErrorKind::Syntax("SPARSE filename required")));
                };
                if name.is_empty() {
                    return Err(error(line, ErrorKind::Layout("empty filename")));
                }
                if name.len() > limits.filename_bytes {
                    return Err(error(line, ErrorKind::Limit("filename bytes")));
                }
                ExtentBacking::Sparse { file_name: name }
            } else if !SPARSE && eq(kind, "FLAT") {
                let [Token::Quoted(name), Token::Bare(offset)] = tail else {
                    return Err(error(
                        line,
                        ErrorKind::Syntax("FLAT filename and offset required"),
                    ));
                };
                if name.is_empty() {
                    return Err(error(line, ErrorKind::Layout("empty filename")));
                }
                if name.len() > limits.filename_bytes {
                    return Err(error(line, ErrorKind::Limit("filename bytes")));
                }
                let offset_bytes = bytes(offset, line)?;
                offset_bytes
                    .checked_add(length)
                    .ok_or(error(line, ErrorKind::Overflow))?;
                ExtentBacking::Flat {
                    file_name: name,
                    offset_bytes,
                }
            } else if !SPARSE && eq(kind, "ZERO") {
                if !tail.is_empty() {
                    return Err(error(line, ErrorKind::Syntax("ZERO has no backing fields")));
                }
                ExtentBacking::Zero
            } else {
                return Err(error(line, ErrorKind::Unsupported("extent type")));
            };
            match create_type {
                Some(CreateType::MonolithicFlat | CreateType::TwoGbMaxExtentFlat) => {
                    if !matches!(
                        backing,
                        ExtentBacking::Flat {
                            offset_bytes: 0,
                            ..
                        }
                    ) {
                        return Err(error(
                            line,
                            ErrorKind::Layout("hosted flat requires FLAT at offset zero"),
                        ));
                    }
                    if create_type == Some(CreateType::MonolithicFlat) && !extents.is_empty() {
                        return Err(error(
                            line,
                            ErrorKind::Layout("monolithic disk requires one extent"),
                        ));
                    }
                    if create_type == Some(CreateType::TwoGbMaxExtentFlat)
                        && length > 2 * 1024 * 1024 * 1024
                    {
                        return Err(error(line, ErrorKind::Layout("split extent exceeds 2 GiB")));
                    }
                }
                Some(CreateType::MonolithicSparse | CreateType::TwoGbMaxExtentSparse) => {
                    if !matches!(backing, ExtentBacking::Sparse { .. }) {
                        return Err(error(
                            line,
                            ErrorKind::Layout("sparse disk requires SPARSE extents"),
                        ));
                    }
                    if create_type == Some(CreateType::MonolithicSparse) && !extents.is_empty() {
                        return Err(error(
                            line,
                            ErrorKind::Layout("monolithic disk requires one extent"),
                        ));
                    }
                    if create_type == Some(CreateType::TwoGbMaxExtentSparse)
                        && length > 2 * 1024 * 1024 * 1024
                    {
                        return Err(error(line, ErrorKind::Layout("split extent exceeds 2 GiB")));
                    }
                }
                Some(CreateType::Custom) => (),
                None => return Err(error(line, ErrorKind::Missing("createType before extents"))),
            }
            let logical_offset = size_bytes;
            size_bytes = size_bytes
                .checked_add(length)
                .ok_or(error(line, ErrorKind::Overflow))?;
            extents.push(Extent {
                access,
                logical_offset,
                size_bytes: length,
                backing,
            });
        }
        version.ok_or(error(0, ErrorKind::Missing("version")))?;
        let parent_cid = parent.ok_or(error(0, ErrorKind::Missing("parentCID")))?;
        let parent = if parent_cid == u32::MAX {
            if parent_hint.is_some() {
                return Err(error(0, ErrorKind::Layout("base has parent hint")));
            }
            None
        } else {
            Some(ParentReference {
                cid: parent_cid,
                file_name_hint: parent_hint
                    .ok_or(error(0, ErrorKind::Missing("parentFileNameHint")))?,
            })
        };
        let cid = content_id.ok_or(error(0, ErrorKind::Missing("CID")))?;
        let create_type = create_type.ok_or(error(0, ErrorKind::Missing("createType")))?;
        if extents.is_empty() {
            return Err(error(0, ErrorKind::Missing("extents")));
        }
        Ok((
            Self {
                cid,
                create_type,
                size_bytes,
                extents,
                metadata,
            },
            parent,
        ))
    }
}

/// Explicit metadata-only sparse subset. The default Descriptor parser stays FLAT/ZERO-only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SparseDescriptor<'a>(Descriptor<'a>);
impl<'a> SparseDescriptor<'a> {
    pub fn parse(input: &'a [u8]) -> Result<Self> {
        Self::parse_with_limits(input, Limits::default())
    }
    /// Accept bounded terminal NUL padding, retaining borrowed input provenance.
    /// All original bytes count toward descriptor_bytes; embedded NULs still reject.
    pub fn parse_with_limits(input: &'a [u8], limits: Limits) -> Result<Self> {
        if input.len() > limits.descriptor_bytes {
            return Err(error(0, ErrorKind::Limit("descriptor bytes")));
        }
        let end = input.iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
        Descriptor::parse_subset::<true, false>(&input[..end], limits).map(|(d, _)| Self(d))
    }
    pub fn cid(&self) -> u32 {
        self.0.cid()
    }
    pub fn create_type(&self) -> CreateType {
        self.0.create_type()
    }
    pub fn size_bytes(&self) -> u64 {
        self.0.size_bytes()
    }
    pub fn extents(&self) -> &[Extent<'a>] {
        self.0.extents()
    }
    pub fn metadata(&self) -> &[Metadata<'a>] {
        self.0.metadata()
    }
    pub(crate) fn same_mapping(&self, other: &Self) -> bool {
        self.cid() == other.cid()
            && self.create_type() == other.create_type()
            && self.extents() == other.extents()
    }
}

/// A CID relationship and untrusted resolver hint; neither is authorization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParentReference<'a> {
    pub cid: u32,
    pub file_name_hint: &'a str,
}

/// Explicit parent-capable metadata syntax. Cannot be passed to SparseDisk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SparseLayerDescriptor<'a> {
    pub(crate) sparse: SparseDescriptor<'a>,
    parent: Option<ParentReference<'a>>,
}
impl<'a> SparseLayerDescriptor<'a> {
    pub fn parse(input: &'a [u8]) -> Result<Self> {
        Self::parse_with_limits(input, Limits::default())
    }
    pub fn parse_with_limits(input: &'a [u8], limits: Limits) -> Result<Self> {
        if input.len() > limits.descriptor_bytes {
            return Err(error(0, ErrorKind::Limit("descriptor bytes")));
        }
        let end = input.iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
        let (descriptor, parent) = Descriptor::parse_subset::<true, true>(&input[..end], limits)?;
        Ok(Self {
            sparse: SparseDescriptor(descriptor),
            parent,
        })
    }
    pub fn parent(&self) -> Option<ParentReference<'a>> {
        self.parent
    }
    pub fn cid(&self) -> u32 {
        self.sparse.cid()
    }
    pub fn size_bytes(&self) -> u64 {
        self.sparse.size_bytes()
    }
    pub fn create_type(&self) -> CreateType {
        self.sparse.create_type()
    }
    pub fn extents(&self) -> &[Extent<'a>] {
        self.sparse.extents()
    }
}
