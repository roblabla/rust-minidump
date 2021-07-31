//! Minidump structure definitions.
//!
//! Types defined here should match those defined in [Microsoft's headers][msdn]. Additionally
//! some [Breakpad][breakpad] and [Crashpad][crashpad] extension types are defined here and should
//! match the definitions from those projects.
//!
//! [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/
//! [breakpad]: https://chromium.googlesource.com/breakpad/breakpad/
//! [crashpad]: https://chromium.googlesource.com/crashpad/crashpad/+/master/README.md
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(clippy::upper_case_acronyms)]

use std::fmt;

use bitflags::bitflags;
use enum_primitive_derive::Primitive;
use scroll::{Endian, Pread, SizeWith};
use smart_default::SmartDefault;

/// An offset from the start of the minidump file.
pub type RVA = u32;
pub type RVA64 = u64;

/// The 4-byte magic number at the start of a minidump file.
///
/// In little endian this spells 'MDMP'.
pub const MINIDUMP_SIGNATURE: u32 = 0x504d444d;

/// The version of the minidump format.
pub const MINIDUMP_VERSION: u32 = 42899;

/// The header at the start of a minidump file.
///
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_header
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct MINIDUMP_HEADER {
    /// This should be [`MINIDUMP_SIGNATURE`][signature].
    ///
    /// [signature]: constant.MINIDUMP_SIGNATURE.html
    pub signature: u32,
    /// This should be [`MINIDUMP_VERSION`][version].
    ///
    /// [version]: constant.MINIDUMP_VERSION.html
    pub version: u32,
    /// The number of streams contained in the stream directory.
    pub stream_count: u32,
    /// The offset to the stream directory within the minidump. This usually points
    /// to immediately after the header. The stream directory is an array containing
    /// `stream_count` [`MINIDUMP_DIRECTORY`][dir] entries.
    ///
    /// [dir]: struct.MINIDUMP_DIRECTORY.html
    pub stream_directory_rva: RVA,
    pub checksum: u32,
    pub time_date_stamp: u32,
    pub flags: u64,
}

/// A location within a minidump file comprised of an offset and a size.
///
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_location_descriptor
#[derive(Debug, Copy, Default, Clone, Pread, SizeWith)]
pub struct MINIDUMP_LOCATION_DESCRIPTOR {
    /// The size of this data.
    pub data_size: u32,
    /// The offset to this data within the minidump file.
    pub rva: RVA,
}

impl From<u8> for MINIDUMP_LOCATION_DESCRIPTOR {
    fn from(_val: u8) -> Self {
        Self::default()
    }
}

/// A range of memory contained within a minidump consisting of a base address and a
/// location descriptor.
///
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_memory_descriptor
#[derive(Debug, Copy, Clone, Default, Pread, SizeWith)]
pub struct MINIDUMP_MEMORY_DESCRIPTOR {
    /// The base address of this memory range from the process.
    pub start_of_memory_range: u64,
    /// The offset and size of the actual bytes of memory contained in this dump.
    pub memory: MINIDUMP_LOCATION_DESCRIPTOR,
}

/// Information about a data stream contained in a minidump file.
///
/// The minidump header contains a pointer to a list of these structs which allows locating
/// specific streams in the dump.
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_directory
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct MINIDUMP_DIRECTORY {
    /// This is usually one of the values in [`MINIDUMP_STREAM_TYPE`][ty] for known stream types,
    /// but user streams can have arbitrary values.
    ///
    /// [ty]: enum.MINIDUMP_STREAM_TYPE.html
    pub stream_type: u32,
    /// The location of the stream contents within the dump.
    pub location: MINIDUMP_LOCATION_DESCRIPTOR,
}

/// The types of known minidump data streams.
///
/// Most of these values are derived from the [Microsoft enum][msdn] of the same name, but
/// the values after `LastReservedStream` are Breakpad and Crashpad extensions.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ne-minidumpapiset-_minidump_stream_type
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum MINIDUMP_STREAM_TYPE {
    /// An unused stream directory entry
    UnusedStream = 0,
    ReservedStream0 = 1,
    ReservedStream1 = 2,
    /// The list of threads from the process
    ///
    /// See [`MINIDUMP_THREAD`].
    ///
    /// Microsoft declares a [`MINIDUMP_THREAD_LIST`][list] struct which is the actual format
    /// of this stream, but it is a variable-length struct so no matching definition is provided
    /// in this crate.
    ///
    /// [list]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_thread_list
    ThreadListStream = 3,
    /// The list of executable modules from the process
    ///
    /// See [`MINIDUMP_MODULE`].
    ///
    /// Microsoft declares a [`MINIDUMP_MODULE_LIST`][list] struct which is the actual format
    /// of this stream, but it is a variable-length struct so no matching definition is provided
    /// in this crate.
    ///
    /// [list]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_module_list
    ModuleListStream = 4,
    /// The list of memory regions from the process contained within this dump
    ///
    /// See [`MINIDUMP_MEMORY_DESCRIPTOR`].
    ///
    /// Microsoft declares a [`MINIDUMP_MEMORY_LIST`][list] struct which is the actual format
    /// of this stream, but it is a variable-length struct so no matching definition is provided
    /// in this crate.
    ///
    /// [list]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_memory_list
    MemoryListStream = 5,
    /// Information about the exception that caused the process to exit
    ///
    /// See [`MINIDUMP_EXCEPTION_STREAM`].
    ExceptionStream = 6,
    /// System information
    ///
    /// See [`MINIDUMP_SYSTEM_INFO`].
    SystemInfoStream = 7,
    ThreadExListStream = 8,
    Memory64ListStream = 9,
    CommentStreamA = 10,
    CommentStreamW = 11,
    HandleDataStream = 12,
    FunctionTable = 13,
    /// The list of executable modules from the process that were unloaded by the time of the crash
    ///
    /// See [`MINIDUMP_UNLOADED_MODULE`].
    ///
    /// Microsoft declares a [`MINIDUMP_UNLOADED_MODULE_LIST`][list] struct which is the actual
    /// format of this stream, but it is a variable-length struct so no matching definition is
    /// in this crate.
    ///
    /// Note that unlike other lists, this one has the newer "extended" header.
    ///
    /// [list]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-minidump_unloaded_module_list
    UnloadedModuleListStream = 14,
    /// Miscellaneous process and system information
    ///
    /// See ['MINIDUMP_MISC_INFO'].
    MiscInfoStream = 15,
    /// Information about memory regions from the process
    ///
    /// See ['MINIDUMP_MEMORY_INFO_LIST'].
    MemoryInfoListStream = 16,
    ThreadInfoListStream = 17,
    HandleOperationListStream = 18,
    TokenStream = 19,
    JavaScriptDataStream = 20,
    SystemMemoryInfoStream = 21,
    ProcessVmCountersStream = 22,
    IptTraceStream = 23,
    /// Names of threads
    ///
    /// See ['MINIDUMP_THREAD_NAME'].
    ThreadNamesStream = 24,
    ceStreamNull = 25,
    ceStreamSystemInfo = 26,
    ceStreamException = 27,
    ceStreamModuleList = 28,
    ceStreamProcessList = 29,
    ceStreamThreadList = 30,
    ceStreamThreadContextList = 31,
    ceStreamThreadCallStackList = 32,
    ceStreamMemoryVirtualList = 33,
    ceStreamMemoryPhysicalList = 34,
    ceStreamBucketParameters = 35,
    ceStreamProcessModuleMap = 36,
    ceStreamDiagnosisList = 37,
    LastReservedStream = 0x0000ffff,
    /* Breakpad extension types.  0x4767 = "Gg" */
    /// Additional process information (Breakpad extension)
    ///
    /// See ['MINIDUMP_BREAKPAD_INFO'].
    BreakpadInfoStream = 0x47670001,
    /// Assertion information (Breakpad extension)
    ///
    /// See ['MINIDUMP_ASSERTION_INFO'].
    AssertionInfoStream = 0x47670002,
    /* These are additional minidump stream values which are specific to
     * the linux breakpad implementation. */
    /// The contents of /proc/cpuinfo from a Linux system
    LinuxCpuInfo = 0x47670003,
    /// The contents of /proc/self/status from a Linux system
    LinuxProcStatus = 0x47670004,
    /// The contents of /etc/lsb-release from a Linux system
    LinuxLsbRelease = 0x47670005,
    /// The contents of /proc/self/cmdline from a Linux system
    LinuxCmdLine = 0x47670006,
    /// The contents of /proc/self/environ from a Linux system
    LinuxEnviron = 0x47670007,
    /// The contents of /proc/self/auxv from a Linux system
    LinuxAuxv = 0x47670008,
    /// The contents of /proc/self/maps from a Linux system
    LinuxMaps = 0x47670009,
    /// Information from the Linux dynamic linker useful for writing core dumps
    ///
    /// See ['DSO_DEBUG_64'] and ['DSO_DEBUG_32'].
    LinuxDsoDebug = 0x4767000A,
    // Crashpad extension types. 0x4350 = "CP"
    /// Crashpad-specific information containing annotations.
    ///
    /// See [`MINIDUMP_CRASHPAD_INFO`].
    CrashpadInfoStream = 0x43500001,

    /// Data from the __DATA,__crash_info section of every module which contains
    /// one that has useful data. Only available on macOS. 0x4D7A = "Mz".
    ///
    /// See ['MINIDUMP_MAC_CRASH_INFO'].
    MozMacosCrashInfoStream = 0x4d7a0001,
}

impl From<MINIDUMP_STREAM_TYPE> for u32 {
    fn from(ty: MINIDUMP_STREAM_TYPE) -> Self {
        ty as u32
    }
}

/// The name of a thread, found in the ThreadNamesStream.
#[derive(Debug, Clone, Default, Pread, SizeWith)]
pub struct MINIDUMP_THREAD_NAME {
    /// The id of the thread.
    pub thread_id: u32,
    /// Where the name of the thread is stored (yes, the legendary RVA64 is real!!).
    pub thread_name_rva: RVA64,
}

/// Information about a single module (executable or shared library) from a minidump
///
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_module
#[derive(Debug, Clone, Default, Pread, SizeWith)]
pub struct MINIDUMP_MODULE {
    /// The base address of the executable image in memory.
    pub base_of_image: u64,
    /// The size of the executable image in memory, in bytes.
    pub size_of_image: u32,
    /// The checksum value from the PE headers.
    pub checksum: u32,
    /// The timestamp value from the PE headers in `time_t` format.
    pub time_date_stamp: u32,
    /// An offset to a length-prefixed UTF-16LE string containing the name of the module.
    pub module_name_rva: RVA,
    /// Version information for this module.
    pub version_info: VS_FIXEDFILEINFO,
    /// The location of a CodeView record describing debug information for this module.
    ///
    /// This should be one of [`CV_INFO_PDB70`][pdb70], [`CV_INFO_PDB20`][pdb20], or
    /// [`CV_INFO_ELF`][elf]. `PDB70` is the most common in practice, describing a standalone PDB
    /// file by way of GUID, age, and PDB filename, and `ELF` is a Breakpad extension for
    /// describing ELF modules with Build IDs.
    ///
    /// See [Matching Debug Information][dbg] for more information.
    ///
    /// [dbg]: http://www.debuginfo.com/articles/debuginfomatch.html
    /// [pdb70]: struct.CV_INFO_PDB70.html
    /// [pdb20]: struct.CV_INFO_PDB20.html
    /// [elf]: struct.CV_INFO_ELF.html
    pub cv_record: MINIDUMP_LOCATION_DESCRIPTOR,
    /// The location of an `IMAGE_DEBUG_MISC` record describing debug information for this module.
    pub misc_record: MINIDUMP_LOCATION_DESCRIPTOR,
    pub reserved0: [u32; 2],
    pub reserved1: [u32; 2],
}

/// Information about a single unloaded module (executable or shared library) from a minidump.
///
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/win32/api/minidumpapiset/ns-minidumpapiset-minidump_unloaded_module
#[derive(Debug, Clone, Default, Pread, SizeWith)]
pub struct MINIDUMP_UNLOADED_MODULE {
    /// The base address of the executable image in memory (when it was loaded).
    pub base_of_image: u64,
    /// The size of the executable image in memory, in bytes.
    pub size_of_image: u32,
    /// The checksum value from the PE headers.
    pub checksum: u32,
    /// The timestamp value from the PE headers in `time_t` format.
    pub time_date_stamp: u32,
    /// An offset to a length-prefixed UTF-16LE string containing the name of the module.
    pub module_name_rva: RVA,
}

/// Version information for a file
///
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/verrsrc/ns-verrsrc-tagvs_fixedfileinfo
#[derive(Debug, Clone, Default, Pread, SizeWith)]
pub struct VS_FIXEDFILEINFO {
    /// Contains the value of `VS_FFI_SIGNATURE`
    pub signature: u32,
    /// Should contain the value of `VS_FFI_STRUCVERSION`
    pub struct_version: u32,
    pub file_version_hi: u32,
    pub file_version_lo: u32,
    pub product_version_hi: u32,
    pub product_version_lo: u32,
    pub file_flags_mask: u32,
    pub file_flags: u32,
    pub file_os: u32,
    pub file_type: u32,
    pub file_subtype: u32,
    pub file_date_hi: u32,
    pub file_date_lo: u32,
}

/// The expected value of `VS_FIXEDFILEINFO.signature`
pub const VS_FFI_SIGNATURE: u32 = 0xfeef04bd;

/// The expected value of `VS_FIXEDFILEINFO.struct_version`
pub const VS_FFI_STRUCVERSION: u32 = 0x00010000;

/// Known values for the `signature` field of CodeView records
///
/// In addition to the two CodeView record formats used for linking
/// to external pdb files it is possible for debugging data to be carried
/// directly in the CodeView record itself.  These signature values will
/// be found in the first 4 bytes of the CodeView record.  Additional values
/// not commonly experienced in the wild are given by ["Microsoft Symbol and
/// Type Information"][sym] section 7.2.  An in-depth description of the CodeView 4.1 format
/// is given by ["Undocumented Windows 2000 Secrets"][win2k], Windows 2000 Debugging Support/
/// Microsoft Symbol File Internals/CodeView Subsections.
///
/// [sym]: http://web.archive.org/web/20070915060650/http://www.x86.org/ftp/manuals/tools/sym.pdf
/// [win2k]: https://dl.acm.org/citation.cfm?id=375734
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum CvSignature {
    /// PDB 2.0 CodeView data: 'NB10': [`CV_INFO_PDB20`]
    Pdb20 = 0x3031424e,
    /// PDB 7.0 CodeView data: 'RSDS': [`CV_INFO_PDB70`]
    Pdb70 = 0x53445352,
    /// ELF Build ID, a Breakpad extension: 'BpEL': [`CV_INFO_ELF`]
    Elf = 0x4270454c,
    /// CodeView 4.10: 'NB09'
    Cv41 = 0x3930424e,
    /// CodeView 5.0: 'NB11'
    Cv50 = 0x3131424e,
}

/// CodeView debug information in the older PDB 2.0 ("NB10") format.
///
/// This struct is defined as variable-length in C with a trailing PDB filename member.
#[derive(Debug, Clone)]
pub struct CV_INFO_PDB20 {
    /// This field will always be [`CvSignature::Pdb20`].
    pub cv_signature: u32,
    pub cv_offset: u32,
    pub signature: u32,
    pub age: u32,
    /// The PDB filename as a zero-terminated byte string
    pub pdb_file_name: Vec<u8>,
}

impl<'a> scroll::ctx::TryFromCtx<'a, Endian> for CV_INFO_PDB20 {
    type Error = scroll::Error;

    fn try_from_ctx(src: &[u8], endian: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        Ok((
            CV_INFO_PDB20 {
                cv_signature: src.gread_with(offset, endian)?,
                cv_offset: src.gread_with(offset, endian)?,
                signature: src.gread_with(offset, endian)?,
                age: src.gread_with(offset, endian)?,
                pdb_file_name: {
                    let size = src.len() - *offset;
                    src.gread_with::<&[u8]>(offset, size)?.to_owned()
                },
            },
            *offset,
        ))
    }
}

/// CodeView debug information in the current PDB 7.0 ("RSDS") format.
///
/// This struct is defined as variable-length in C with a trailing PDB filename member.
#[derive(Debug, Clone)]
pub struct CV_INFO_PDB70 {
    /// This will always be [`CvSignature::Pdb70`]
    pub cv_signature: u32,
    /// A unique identifer for a module created on first build.
    pub signature: GUID,
    /// A counter, incremented for each rebuild that updates the PDB file.
    pub age: u32,
    /// The PDB filename as a zero-terminated byte string
    pub pdb_file_name: Vec<u8>,
}

impl<'a> scroll::ctx::TryFromCtx<'a, Endian> for CV_INFO_PDB70 {
    type Error = scroll::Error;

    fn try_from_ctx(src: &[u8], endian: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        Ok((
            CV_INFO_PDB70 {
                cv_signature: src.gread_with(offset, endian)?,
                signature: src.gread_with(offset, endian)?,
                age: src.gread_with(offset, endian)?,
                pdb_file_name: {
                    let size = src.len() - *offset;
                    src.gread_with::<&[u8]>(offset, size)?.to_owned()
                },
            },
            *offset,
        ))
    }
}

/// A GUID as specified in Rpcdce.h
///
/// Matches the [Microsoft struct][msdn] of the same name.
///
/// # Display
///
/// There are two `Display` implementations for GUIDs. The regular formatting is lowercase with
/// hyphens. The alternate formatting used with `#` is the symbol server format (uppercase without
/// hyphens).
///
/// ```
/// use minidump_common::format::GUID;
///
/// let guid = GUID { data1: 10, data2: 11, data3: 12, data4: [1,2,3,4,5,6,7,8]};
///
/// // default formatting
/// assert_eq!("0000000a-000b-000c-0102-030405060708", guid.to_string());
///
/// // symbol server formatting
/// assert_eq!("0000000A000B000C0102030405060708", format!("{:#}", guid));
/// ```
///
/// [msdn]: https://msdn.microsoft.com/en-us/library/windows/desktop/aa373931(v=vs.85).aspx
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pread, SizeWith)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl fmt::Display for GUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NB: This formatting is not endianness aware. GUIDs read from LE minidumps are printed
        // with reversed fields.
        if f.alternate() {
            write!(
                f,
                "{:08X}{:04X}{:04X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
                self.data1,
                self.data2,
                self.data3,
                self.data4[0],
                self.data4[1],
                self.data4[2],
                self.data4[3],
                self.data4[4],
                self.data4[5],
                self.data4[6],
                self.data4[7],
            )
        } else {
            write!(
                f,
                "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                self.data1,
                self.data2,
                self.data3,
                self.data4[0],
                self.data4[1],
                self.data4[2],
                self.data4[3],
                self.data4[4],
                self.data4[5],
                self.data4[6],
                self.data4[7],
            )
        }
    }
}

/// An ELF Build ID.
///
/// Modern ELF toolchains insert a "[build id][buildid]" into the ELF headers that typically
/// contains a hash of some ELF headers and sections to uniquely identify a binary. The Build ID
/// is allowed to be an arbitrary number of bytes however, and [GNU binutils allows creating
/// ELF binaries with Build IDs of various formats][binutils].
///
/// [buildid]: https://access.redhat.com/documentation/en-us/red_hat_enterprise_linux/6/html/developer_guide/compiling-build-id
/// [binutils]: https://sourceware.org/binutils/docs-2.26/ld/Options.html#index-g_t_002d_002dbuild_002did-292
#[derive(Debug, Clone)]
pub struct CV_INFO_ELF {
    /// This will always be [`CvSignature::Elf`]
    pub cv_signature: u32,
    /// The build id, a variable number of bytes
    pub build_id: Vec<u8>,
}

impl<'a> scroll::ctx::TryFromCtx<'a, Endian> for CV_INFO_ELF {
    type Error = scroll::Error;

    fn try_from_ctx(src: &'a [u8], endian: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        Ok((
            CV_INFO_ELF {
                cv_signature: src.gread_with(offset, endian)?,
                build_id: {
                    let size = src.len() - *offset;
                    src.gread_with::<&[u8]>(offset, size)?.to_owned()
                },
            },
            *offset,
        ))
    }
}

/// Obsolete debug record type defined in WinNT.h.
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct IMAGE_DEBUG_MISC {
    pub data_type: u32,
    pub length: u32,
    pub unicode: u8,
    pub reserved: [u8; 3],
    pub data: [u8; 1],
}

/// Information about a single thread from a minidump
///
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_thread
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct MINIDUMP_THREAD {
    /// The identifier of this thread
    pub thread_id: u32,
    /// The suspend count for this thread
    ///
    /// If greater than zero, the thread is suspended.
    pub suspend_count: u32,
    /// The priority class of the thread
    ///
    /// See [Scheduling Priorities][msdn] on MSDN.
    ///
    /// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/ProcThread/scheduling-priorities
    pub priority_class: u32,
    /// The priority level of the thread
    pub priority: u32,
    /// The thread environment block
    pub teb: u64,
    /// The location and base address of this thread's stack memory
    pub stack: MINIDUMP_MEMORY_DESCRIPTOR,
    /// The location of a CPU-specific `CONTEXT_` struct for this thread's CPU context
    pub thread_context: MINIDUMP_LOCATION_DESCRIPTOR,
}

/// Information about the exception that caused the process to terminate.
///
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-minidump_exception_stream
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct MINIDUMP_EXCEPTION_STREAM {
    /// The identifier of the thread that encountered the exception.
    pub thread_id: u32,
    pub __align: u32,
    /// Detailed information about the exception encountered.
    pub exception_record: MINIDUMP_EXCEPTION,
    /// The offset of a CPU context record from the time the thread encountered the exception.
    ///
    /// The actual data will be one of the `CONTEXT_*` structs defined here.
    pub thread_context: MINIDUMP_LOCATION_DESCRIPTOR,
}

/// Detailed information about an exception.
///
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_exception
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct MINIDUMP_EXCEPTION {
    /// The reason the exception occurred.
    ///
    /// Possible values are in the [`ExceptionCodeWindows`], [`WinErrorWindows`],
    /// [`NtStatusWindows`], [`ExceptionCodeLinux`], and [`ExceptionCodeMac`] enums.
    pub exception_code: u32,
    /// Flags related to the exception.
    ///
    /// On Windows this is 1 for noncontinuable exceptions and 0 otherwise. For Breakpad-produced
    /// minidumps on macOS this field is used to store additional exception information.
    pub exception_flags: u32,
    /// The address of an associated [`MINIDUMP_EXCEPTION`] for a nested exception.
    ///
    /// This address is in the minidump producing host's memory.
    pub exception_record: u64,
    /// The address where the exception occurred.
    ///
    /// For Breakpad-produced minidumps on macOS this is the exception subcode, which is
    /// typically the address.
    pub exception_address: u64,
    /// The number of valid elements in [`MINIDUMP_EXCEPTION::exception_information`].
    pub number_parameters: u32,
    pub __align: u32,
    /// An array of additional arguments that describe the exception.
    ///
    /// For most exception codes the array elements are undefined, but for access violations
    /// the array will contain two elements: a read/write flag in the first element and
    /// the virtual address whose access caused the exception in the second element.
    pub exception_information: [u64; 15], // EXCEPTION_MAXIMUM_PARAMETERS
}

/// Values for [`MINIDUMP_EXCEPTION::exception_code`] for crashes on Windows
///
/// These values come from WinBase.h and WinNT.h with a few additions.
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeWindows {
    EXCEPTION_GUARD_PAGE = 0x80000001u32,
    EXCEPTION_DATATYPE_MISALIGNMENT = 0x80000002,
    EXCEPTION_BREAKPOINT = 0x80000003,
    EXCEPTION_SINGLE_STEP = 0x80000004,
    EXCEPTION_ACCESS_VIOLATION = 0xc0000005,
    EXCEPTION_IN_PAGE_ERROR = 0xc0000006,
    EXCEPTION_INVALID_HANDLE = 0xc0000008,
    EXCEPTION_ILLEGAL_INSTRUCTION = 0xc000001d,
    EXCEPTION_NONCONTINUABLE_EXCEPTION = 0xc0000025,
    EXCEPTION_INVALID_DISPOSITION = 0xc0000026,
    EXCEPTION_BOUNDS_EXCEEDED = 0xc000008c,
    EXCEPTION_FLT_DENORMAL_OPERAND = 0xc000008d,
    EXCEPTION_FLT_DIVIDE_BY_ZERO = 0xc000008e,
    EXCEPTION_FLT_INEXACT_RESULT = 0xc000008f,
    EXCEPTION_FLT_INVALID_OPERATION = 0xc0000090,
    EXCEPTION_FLT_OVERFLOW = 0xc0000091,
    EXCEPTION_FLT_STACK_CHECK = 0xc0000092,
    EXCEPTION_FLT_UNDERFLOW = 0xc0000093,
    EXCEPTION_INT_DIVIDE_BY_ZERO = 0xc0000094,
    EXCEPTION_INT_OVERFLOW = 0xc0000095,
    EXCEPTION_PRIV_INSTRUCTION = 0xc0000096,
    EXCEPTION_STACK_OVERFLOW = 0xc00000fd,
    EXCEPTION_POSSIBLE_DEADLOCK = 0xc0000194,
    /// Exception thrown by Chromium allocators to indicate OOM
    ///
    /// See base/process/memory.h in Chromium for rationale.
    OUT_OF_MEMORY = 0xe0000008,
    /// Per <http://support.microsoft.com/kb/185294>, generated by Visual C++ compiler
    UNHANDLED_CPP_EXCEPTION = 0xe06d7363,
    /// Fake exception code used by Crashpad
    SIMULATED = 0x0517a7ed,
}

/// Values for [`MINIDUMP_EXCEPTION::exception_code`] for crashes on Windows
///
/// The values were generated from from winerror.h in the Windows 10 SDK
/// (version 10.0.19041.0) using the following script:
/// ```sh
/// egrep -o '#define (ERROR_|RPC_[ESX]_)[A-Z_0-9]+\s+[0-9]+L' winerror.h \
///   | tr -d '\r' \
///   | sed -r 's@#define ((ERROR_|RPC_[ESX]_)[A-Z_0-9]+)\s+([0-9]+)L@\3 \1@' \
///   | sort -n \
///   | sed -r 's@([0-9]+) ([A-Z_0-9]+)@    \2 = \L\1,@'
/// ```
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum WinErrorWindows {
    ERROR_SUCCESS = 0,
    ERROR_INVALID_FUNCTION = 1,
    ERROR_FILE_NOT_FOUND = 2,
    ERROR_PATH_NOT_FOUND = 3,
    ERROR_TOO_MANY_OPEN_FILES = 4,
    ERROR_ACCESS_DENIED = 5,
    ERROR_INVALID_HANDLE = 6,
    ERROR_ARENA_TRASHED = 7,
    ERROR_NOT_ENOUGH_MEMORY = 8,
    ERROR_INVALID_BLOCK = 9,
    ERROR_BAD_ENVIRONMENT = 10,
    ERROR_BAD_FORMAT = 11,
    ERROR_INVALID_ACCESS = 12,
    ERROR_INVALID_DATA = 13,
    ERROR_OUTOFMEMORY = 14,
    ERROR_INVALID_DRIVE = 15,
    ERROR_CURRENT_DIRECTORY = 16,
    ERROR_NOT_SAME_DEVICE = 17,
    ERROR_NO_MORE_FILES = 18,
    ERROR_WRITE_PROTECT = 19,
    ERROR_BAD_UNIT = 20,
    ERROR_NOT_READY = 21,
    ERROR_BAD_COMMAND = 22,
    ERROR_CRC = 23,
    ERROR_BAD_LENGTH = 24,
    ERROR_SEEK = 25,
    ERROR_NOT_DOS_DISK = 26,
    ERROR_SECTOR_NOT_FOUND = 27,
    ERROR_OUT_OF_PAPER = 28,
    ERROR_WRITE_FAULT = 29,
    ERROR_READ_FAULT = 30,
    ERROR_GEN_FAILURE = 31,
    ERROR_SHARING_VIOLATION = 32,
    ERROR_LOCK_VIOLATION = 33,
    ERROR_WRONG_DISK = 34,
    ERROR_SHARING_BUFFER_EXCEEDED = 36,
    ERROR_HANDLE_EOF = 38,
    ERROR_HANDLE_DISK_FULL = 39,
    ERROR_NOT_SUPPORTED = 50,
    ERROR_REM_NOT_LIST = 51,
    ERROR_DUP_NAME = 52,
    ERROR_BAD_NETPATH = 53,
    ERROR_NETWORK_BUSY = 54,
    ERROR_DEV_NOT_EXIST = 55,
    ERROR_TOO_MANY_CMDS = 56,
    ERROR_ADAP_HDW_ERR = 57,
    ERROR_BAD_NET_RESP = 58,
    ERROR_UNEXP_NET_ERR = 59,
    ERROR_BAD_REM_ADAP = 60,
    ERROR_PRINTQ_FULL = 61,
    ERROR_NO_SPOOL_SPACE = 62,
    ERROR_PRINT_CANCELLED = 63,
    ERROR_NETNAME_DELETED = 64,
    ERROR_NETWORK_ACCESS_DENIED = 65,
    ERROR_BAD_DEV_TYPE = 66,
    ERROR_BAD_NET_NAME = 67,
    ERROR_TOO_MANY_NAMES = 68,
    ERROR_TOO_MANY_SESS = 69,
    ERROR_SHARING_PAUSED = 70,
    ERROR_REQ_NOT_ACCEP = 71,
    ERROR_REDIR_PAUSED = 72,
    ERROR_FILE_EXISTS = 80,
    ERROR_CANNOT_MAKE = 82,
    ERROR_FAIL_I24 = 83,
    ERROR_OUT_OF_STRUCTURES = 84,
    ERROR_ALREADY_ASSIGNED = 85,
    ERROR_INVALID_PASSWORD = 86,
    ERROR_INVALID_PARAMETER = 87,
    ERROR_NET_WRITE_FAULT = 88,
    ERROR_NO_PROC_SLOTS = 89,
    ERROR_TOO_MANY_SEMAPHORES = 100,
    ERROR_EXCL_SEM_ALREADY_OWNED = 101,
    ERROR_SEM_IS_SET = 102,
    ERROR_TOO_MANY_SEM_REQUESTS = 103,
    ERROR_INVALID_AT_INTERRUPT_TIME = 104,
    ERROR_SEM_OWNER_DIED = 105,
    ERROR_SEM_USER_LIMIT = 106,
    ERROR_DISK_CHANGE = 107,
    ERROR_DRIVE_LOCKED = 108,
    ERROR_BROKEN_PIPE = 109,
    ERROR_OPEN_FAILED = 110,
    ERROR_BUFFER_OVERFLOW = 111,
    ERROR_DISK_FULL = 112,
    ERROR_NO_MORE_SEARCH_HANDLES = 113,
    ERROR_INVALID_TARGET_HANDLE = 114,
    ERROR_INVALID_CATEGORY = 117,
    ERROR_INVALID_VERIFY_SWITCH = 118,
    ERROR_BAD_DRIVER_LEVEL = 119,
    ERROR_CALL_NOT_IMPLEMENTED = 120,
    ERROR_SEM_TIMEOUT = 121,
    ERROR_INSUFFICIENT_BUFFER = 122,
    ERROR_INVALID_NAME = 123,
    ERROR_INVALID_LEVEL = 124,
    ERROR_NO_VOLUME_LABEL = 125,
    ERROR_MOD_NOT_FOUND = 126,
    ERROR_PROC_NOT_FOUND = 127,
    ERROR_WAIT_NO_CHILDREN = 128,
    ERROR_CHILD_NOT_COMPLETE = 129,
    ERROR_DIRECT_ACCESS_HANDLE = 130,
    ERROR_NEGATIVE_SEEK = 131,
    ERROR_SEEK_ON_DEVICE = 132,
    ERROR_IS_JOIN_TARGET = 133,
    ERROR_IS_JOINED = 134,
    ERROR_IS_SUBSTED = 135,
    ERROR_NOT_JOINED = 136,
    ERROR_NOT_SUBSTED = 137,
    ERROR_JOIN_TO_JOIN = 138,
    ERROR_SUBST_TO_SUBST = 139,
    ERROR_JOIN_TO_SUBST = 140,
    ERROR_SUBST_TO_JOIN = 141,
    ERROR_BUSY_DRIVE = 142,
    ERROR_SAME_DRIVE = 143,
    ERROR_DIR_NOT_ROOT = 144,
    ERROR_DIR_NOT_EMPTY = 145,
    ERROR_IS_SUBST_PATH = 146,
    ERROR_IS_JOIN_PATH = 147,
    ERROR_PATH_BUSY = 148,
    ERROR_IS_SUBST_TARGET = 149,
    ERROR_SYSTEM_TRACE = 150,
    ERROR_INVALID_EVENT_COUNT = 151,
    ERROR_TOO_MANY_MUXWAITERS = 152,
    ERROR_INVALID_LIST_FORMAT = 153,
    ERROR_LABEL_TOO_LONG = 154,
    ERROR_TOO_MANY_TCBS = 155,
    ERROR_SIGNAL_REFUSED = 156,
    ERROR_DISCARDED = 157,
    ERROR_NOT_LOCKED = 158,
    ERROR_BAD_THREADID_ADDR = 159,
    ERROR_BAD_ARGUMENTS = 160,
    ERROR_BAD_PATHNAME = 161,
    ERROR_SIGNAL_PENDING = 162,
    ERROR_MAX_THRDS_REACHED = 164,
    ERROR_LOCK_FAILED = 167,
    ERROR_BUSY = 170,
    ERROR_DEVICE_SUPPORT_IN_PROGRESS = 171,
    ERROR_CANCEL_VIOLATION = 173,
    ERROR_ATOMIC_LOCKS_NOT_SUPPORTED = 174,
    ERROR_INVALID_SEGMENT_NUMBER = 180,
    ERROR_INVALID_ORDINAL = 182,
    ERROR_ALREADY_EXISTS = 183,
    ERROR_INVALID_FLAG_NUMBER = 186,
    ERROR_SEM_NOT_FOUND = 187,
    ERROR_INVALID_STARTING_CODESEG = 188,
    ERROR_INVALID_STACKSEG = 189,
    ERROR_INVALID_MODULETYPE = 190,
    ERROR_INVALID_EXE_SIGNATURE = 191,
    ERROR_EXE_MARKED_INVALID = 192,
    ERROR_BAD_EXE_FORMAT = 193,
    ERROR_INVALID_MINALLOCSIZE = 195,
    ERROR_DYNLINK_FROM_INVALID_RING = 196,
    ERROR_IOPL_NOT_ENABLED = 197,
    ERROR_INVALID_SEGDPL = 198,
    ERROR_RING2SEG_MUST_BE_MOVABLE = 200,
    ERROR_RELOC_CHAIN_XEEDS_SEGLIM = 201,
    ERROR_INFLOOP_IN_RELOC_CHAIN = 202,
    ERROR_ENVVAR_NOT_FOUND = 203,
    ERROR_NO_SIGNAL_SENT = 205,
    ERROR_FILENAME_EXCED_RANGE = 206,
    ERROR_RING2_STACK_IN_USE = 207,
    ERROR_META_EXPANSION_TOO_LONG = 208,
    ERROR_INVALID_SIGNAL_NUMBER = 209,
    ERROR_THREAD_1_INACTIVE = 210,
    ERROR_LOCKED = 212,
    ERROR_TOO_MANY_MODULES = 214,
    ERROR_NESTING_NOT_ALLOWED = 215,
    ERROR_EXE_MACHINE_TYPE_MISMATCH = 216,
    ERROR_EXE_CANNOT_MODIFY_SIGNED_BINARY = 217,
    ERROR_EXE_CANNOT_MODIFY_STRONG_SIGNED_BINARY = 218,
    ERROR_FILE_CHECKED_OUT = 220,
    ERROR_CHECKOUT_REQUIRED = 221,
    ERROR_BAD_FILE_TYPE = 222,
    ERROR_FILE_TOO_LARGE = 223,
    ERROR_FORMS_AUTH_REQUIRED = 224,
    ERROR_VIRUS_INFECTED = 225,
    ERROR_VIRUS_DELETED = 226,
    ERROR_PIPE_LOCAL = 229,
    ERROR_BAD_PIPE = 230,
    ERROR_PIPE_BUSY = 231,
    ERROR_NO_DATA = 232,
    ERROR_PIPE_NOT_CONNECTED = 233,
    ERROR_MORE_DATA = 234,
    ERROR_NO_WORK_DONE = 235,
    ERROR_VC_DISCONNECTED = 240,
    ERROR_INVALID_EA_NAME = 254,
    ERROR_EA_LIST_INCONSISTENT = 255,
    ERROR_NO_MORE_ITEMS = 259,
    ERROR_CANNOT_COPY = 266,
    ERROR_DIRECTORY = 267,
    ERROR_EAS_DIDNT_FIT = 275,
    ERROR_EA_FILE_CORRUPT = 276,
    ERROR_EA_TABLE_FULL = 277,
    ERROR_INVALID_EA_HANDLE = 278,
    ERROR_EAS_NOT_SUPPORTED = 282,
    ERROR_NOT_OWNER = 288,
    ERROR_TOO_MANY_POSTS = 298,
    ERROR_PARTIAL_COPY = 299,
    ERROR_OPLOCK_NOT_GRANTED = 300,
    ERROR_INVALID_OPLOCK_PROTOCOL = 301,
    ERROR_DISK_TOO_FRAGMENTED = 302,
    ERROR_DELETE_PENDING = 303,
    ERROR_INCOMPATIBLE_WITH_GLOBAL_SHORT_NAME_REGISTRY_SETTING = 304,
    ERROR_SHORT_NAMES_NOT_ENABLED_ON_VOLUME = 305,
    ERROR_SECURITY_STREAM_IS_INCONSISTENT = 306,
    ERROR_INVALID_LOCK_RANGE = 307,
    ERROR_IMAGE_SUBSYSTEM_NOT_PRESENT = 308,
    ERROR_NOTIFICATION_GUID_ALREADY_DEFINED = 309,
    ERROR_INVALID_EXCEPTION_HANDLER = 310,
    ERROR_DUPLICATE_PRIVILEGES = 311,
    ERROR_NO_RANGES_PROCESSED = 312,
    ERROR_NOT_ALLOWED_ON_SYSTEM_FILE = 313,
    ERROR_DISK_RESOURCES_EXHAUSTED = 314,
    ERROR_INVALID_TOKEN = 315,
    ERROR_DEVICE_FEATURE_NOT_SUPPORTED = 316,
    ERROR_MR_MID_NOT_FOUND = 317,
    ERROR_SCOPE_NOT_FOUND = 318,
    ERROR_UNDEFINED_SCOPE = 319,
    ERROR_INVALID_CAP = 320,
    ERROR_DEVICE_UNREACHABLE = 321,
    ERROR_DEVICE_NO_RESOURCES = 322,
    ERROR_DATA_CHECKSUM_ERROR = 323,
    ERROR_INTERMIXED_KERNEL_EA_OPERATION = 324,
    ERROR_FILE_LEVEL_TRIM_NOT_SUPPORTED = 326,
    ERROR_OFFSET_ALIGNMENT_VIOLATION = 327,
    ERROR_INVALID_FIELD_IN_PARAMETER_LIST = 328,
    ERROR_OPERATION_IN_PROGRESS = 329,
    ERROR_BAD_DEVICE_PATH = 330,
    ERROR_TOO_MANY_DESCRIPTORS = 331,
    ERROR_SCRUB_DATA_DISABLED = 332,
    ERROR_NOT_REDUNDANT_STORAGE = 333,
    ERROR_RESIDENT_FILE_NOT_SUPPORTED = 334,
    ERROR_COMPRESSED_FILE_NOT_SUPPORTED = 335,
    ERROR_DIRECTORY_NOT_SUPPORTED = 336,
    ERROR_NOT_READ_FROM_COPY = 337,
    ERROR_FT_WRITE_FAILURE = 338,
    ERROR_FT_DI_SCAN_REQUIRED = 339,
    ERROR_INVALID_KERNEL_INFO_VERSION = 340,
    ERROR_INVALID_PEP_INFO_VERSION = 341,
    ERROR_OBJECT_NOT_EXTERNALLY_BACKED = 342,
    ERROR_EXTERNAL_BACKING_PROVIDER_UNKNOWN = 343,
    ERROR_COMPRESSION_NOT_BENEFICIAL = 344,
    ERROR_STORAGE_TOPOLOGY_ID_MISMATCH = 345,
    ERROR_BLOCKED_BY_PARENTAL_CONTROLS = 346,
    ERROR_BLOCK_TOO_MANY_REFERENCES = 347,
    ERROR_MARKED_TO_DISALLOW_WRITES = 348,
    ERROR_ENCLAVE_FAILURE = 349,
    ERROR_FAIL_NOACTION_REBOOT = 350,
    ERROR_FAIL_SHUTDOWN = 351,
    ERROR_FAIL_RESTART = 352,
    ERROR_MAX_SESSIONS_REACHED = 353,
    ERROR_NETWORK_ACCESS_DENIED_EDP = 354,
    ERROR_DEVICE_HINT_NAME_BUFFER_TOO_SMALL = 355,
    ERROR_EDP_POLICY_DENIES_OPERATION = 356,
    ERROR_EDP_DPL_POLICY_CANT_BE_SATISFIED = 357,
    ERROR_CLOUD_FILE_SYNC_ROOT_METADATA_CORRUPT = 358,
    ERROR_DEVICE_IN_MAINTENANCE = 359,
    ERROR_NOT_SUPPORTED_ON_DAX = 360,
    ERROR_DAX_MAPPING_EXISTS = 361,
    ERROR_CLOUD_FILE_PROVIDER_NOT_RUNNING = 362,
    ERROR_CLOUD_FILE_METADATA_CORRUPT = 363,
    ERROR_CLOUD_FILE_METADATA_TOO_LARGE = 364,
    ERROR_CLOUD_FILE_PROPERTY_BLOB_TOO_LARGE = 365,
    ERROR_CLOUD_FILE_PROPERTY_BLOB_CHECKSUM_MISMATCH = 366,
    ERROR_CHILD_PROCESS_BLOCKED = 367,
    ERROR_STORAGE_LOST_DATA_PERSISTENCE = 368,
    ERROR_FILE_SYSTEM_VIRTUALIZATION_UNAVAILABLE = 369,
    ERROR_FILE_SYSTEM_VIRTUALIZATION_METADATA_CORRUPT = 370,
    ERROR_FILE_SYSTEM_VIRTUALIZATION_BUSY = 371,
    ERROR_FILE_SYSTEM_VIRTUALIZATION_PROVIDER_UNKNOWN = 372,
    ERROR_GDI_HANDLE_LEAK = 373,
    ERROR_CLOUD_FILE_TOO_MANY_PROPERTY_BLOBS = 374,
    ERROR_CLOUD_FILE_PROPERTY_VERSION_NOT_SUPPORTED = 375,
    ERROR_NOT_A_CLOUD_FILE = 376,
    ERROR_CLOUD_FILE_NOT_IN_SYNC = 377,
    ERROR_CLOUD_FILE_ALREADY_CONNECTED = 378,
    ERROR_CLOUD_FILE_NOT_SUPPORTED = 379,
    ERROR_CLOUD_FILE_INVALID_REQUEST = 380,
    ERROR_CLOUD_FILE_READ_ONLY_VOLUME = 381,
    ERROR_CLOUD_FILE_CONNECTED_PROVIDER_ONLY = 382,
    ERROR_CLOUD_FILE_VALIDATION_FAILED = 383,
    ERROR_SMB1_NOT_AVAILABLE = 384,
    ERROR_FILE_SYSTEM_VIRTUALIZATION_INVALID_OPERATION = 385,
    ERROR_CLOUD_FILE_AUTHENTICATION_FAILED = 386,
    ERROR_CLOUD_FILE_INSUFFICIENT_RESOURCES = 387,
    ERROR_CLOUD_FILE_NETWORK_UNAVAILABLE = 388,
    ERROR_CLOUD_FILE_UNSUCCESSFUL = 389,
    ERROR_CLOUD_FILE_NOT_UNDER_SYNC_ROOT = 390,
    ERROR_CLOUD_FILE_IN_USE = 391,
    ERROR_CLOUD_FILE_PINNED = 392,
    ERROR_CLOUD_FILE_REQUEST_ABORTED = 393,
    ERROR_CLOUD_FILE_PROPERTY_CORRUPT = 394,
    ERROR_CLOUD_FILE_ACCESS_DENIED = 395,
    ERROR_CLOUD_FILE_INCOMPATIBLE_HARDLINKS = 396,
    ERROR_CLOUD_FILE_PROPERTY_LOCK_CONFLICT = 397,
    ERROR_CLOUD_FILE_REQUEST_CANCELED = 398,
    ERROR_EXTERNAL_SYSKEY_NOT_SUPPORTED = 399,
    ERROR_THREAD_MODE_ALREADY_BACKGROUND = 400,
    ERROR_THREAD_MODE_NOT_BACKGROUND = 401,
    ERROR_PROCESS_MODE_ALREADY_BACKGROUND = 402,
    ERROR_PROCESS_MODE_NOT_BACKGROUND = 403,
    ERROR_CLOUD_FILE_PROVIDER_TERMINATED = 404,
    ERROR_NOT_A_CLOUD_SYNC_ROOT = 405,
    ERROR_FILE_PROTECTED_UNDER_DPL = 406,
    ERROR_VOLUME_NOT_CLUSTER_ALIGNED = 407,
    ERROR_NO_PHYSICALLY_ALIGNED_FREE_SPACE_FOUND = 408,
    ERROR_APPX_FILE_NOT_ENCRYPTED = 409,
    ERROR_RWRAW_ENCRYPTED_FILE_NOT_ENCRYPTED = 410,
    ERROR_RWRAW_ENCRYPTED_INVALID_EDATAINFO_FILEOFFSET = 411,
    ERROR_RWRAW_ENCRYPTED_INVALID_EDATAINFO_FILERANGE = 412,
    ERROR_RWRAW_ENCRYPTED_INVALID_EDATAINFO_PARAMETER = 413,
    ERROR_LINUX_SUBSYSTEM_NOT_PRESENT = 414,
    ERROR_FT_READ_FAILURE = 415,
    ERROR_STORAGE_RESERVE_ID_INVALID = 416,
    ERROR_STORAGE_RESERVE_DOES_NOT_EXIST = 417,
    ERROR_STORAGE_RESERVE_ALREADY_EXISTS = 418,
    ERROR_STORAGE_RESERVE_NOT_EMPTY = 419,
    ERROR_NOT_A_DAX_VOLUME = 420,
    ERROR_NOT_DAX_MAPPABLE = 421,
    ERROR_TIME_SENSITIVE_THREAD = 422,
    ERROR_DPL_NOT_SUPPORTED_FOR_USER = 423,
    ERROR_CASE_DIFFERING_NAMES_IN_DIR = 424,
    ERROR_FILE_NOT_SUPPORTED = 425,
    ERROR_CLOUD_FILE_REQUEST_TIMEOUT = 426,
    ERROR_NO_TASK_QUEUE = 427,
    ERROR_SRC_SRV_DLL_LOAD_FAILED = 428,
    ERROR_NOT_SUPPORTED_WITH_BTT = 429,
    ERROR_ENCRYPTION_DISABLED = 430,
    ERROR_ENCRYPTING_METADATA_DISALLOWED = 431,
    ERROR_CANT_CLEAR_ENCRYPTION_FLAG = 432,
    ERROR_NO_SUCH_DEVICE = 433,
    ERROR_CLOUD_FILE_DEHYDRATION_DISALLOWED = 434,
    ERROR_FILE_SNAP_IN_PROGRESS = 435,
    ERROR_FILE_SNAP_USER_SECTION_NOT_SUPPORTED = 436,
    ERROR_FILE_SNAP_MODIFY_NOT_SUPPORTED = 437,
    ERROR_FILE_SNAP_IO_NOT_COORDINATED = 438,
    ERROR_FILE_SNAP_UNEXPECTED_ERROR = 439,
    ERROR_FILE_SNAP_INVALID_PARAMETER = 440,
    ERROR_UNSATISFIED_DEPENDENCIES = 441,
    ERROR_CASE_SENSITIVE_PATH = 442,
    ERROR_UNEXPECTED_NTCACHEMANAGER_ERROR = 443,
    ERROR_LINUX_SUBSYSTEM_UPDATE_REQUIRED = 444,
    ERROR_DLP_POLICY_WARNS_AGAINST_OPERATION = 445,
    ERROR_DLP_POLICY_DENIES_OPERATION = 446,
    ERROR_DLP_POLICY_SILENTLY_FAIL = 449,
    ERROR_CAPAUTHZ_NOT_DEVUNLOCKED = 450,
    ERROR_CAPAUTHZ_CHANGE_TYPE = 451,
    ERROR_CAPAUTHZ_NOT_PROVISIONED = 452,
    ERROR_CAPAUTHZ_NOT_AUTHORIZED = 453,
    ERROR_CAPAUTHZ_NO_POLICY = 454,
    ERROR_CAPAUTHZ_DB_CORRUPTED = 455,
    ERROR_CAPAUTHZ_SCCD_INVALID_CATALOG = 456,
    ERROR_CAPAUTHZ_SCCD_NO_AUTH_ENTITY = 457,
    ERROR_CAPAUTHZ_SCCD_PARSE_ERROR = 458,
    ERROR_CAPAUTHZ_SCCD_DEV_MODE_REQUIRED = 459,
    ERROR_CAPAUTHZ_SCCD_NO_CAPABILITY_MATCH = 460,
    ERROR_CIMFS_IMAGE_CORRUPT = 470,
    ERROR_PNP_QUERY_REMOVE_DEVICE_TIMEOUT = 480,
    ERROR_PNP_QUERY_REMOVE_RELATED_DEVICE_TIMEOUT = 481,
    ERROR_PNP_QUERY_REMOVE_UNRELATED_DEVICE_TIMEOUT = 482,
    ERROR_DEVICE_HARDWARE_ERROR = 483,
    ERROR_INVALID_ADDRESS = 487,
    ERROR_HAS_SYSTEM_CRITICAL_FILES = 488,
    ERROR_USER_PROFILE_LOAD = 500,
    ERROR_ARITHMETIC_OVERFLOW = 534,
    ERROR_PIPE_CONNECTED = 535,
    ERROR_PIPE_LISTENING = 536,
    ERROR_VERIFIER_STOP = 537,
    ERROR_ABIOS_ERROR = 538,
    ERROR_WX86_WARNING = 539,
    ERROR_WX86_ERROR = 540,
    ERROR_TIMER_NOT_CANCELED = 541,
    ERROR_UNWIND = 542,
    ERROR_BAD_STACK = 543,
    ERROR_INVALID_UNWIND_TARGET = 544,
    ERROR_INVALID_PORT_ATTRIBUTES = 545,
    ERROR_PORT_MESSAGE_TOO_LONG = 546,
    ERROR_INVALID_QUOTA_LOWER = 547,
    ERROR_DEVICE_ALREADY_ATTACHED = 548,
    ERROR_INSTRUCTION_MISALIGNMENT = 549,
    ERROR_PROFILING_NOT_STARTED = 550,
    ERROR_PROFILING_NOT_STOPPED = 551,
    ERROR_COULD_NOT_INTERPRET = 552,
    ERROR_PROFILING_AT_LIMIT = 553,
    ERROR_CANT_WAIT = 554,
    ERROR_CANT_TERMINATE_SELF = 555,
    ERROR_UNEXPECTED_MM_CREATE_ERR = 556,
    ERROR_UNEXPECTED_MM_MAP_ERROR = 557,
    ERROR_UNEXPECTED_MM_EXTEND_ERR = 558,
    ERROR_BAD_FUNCTION_TABLE = 559,
    ERROR_NO_GUID_TRANSLATION = 560,
    ERROR_INVALID_LDT_SIZE = 561,
    ERROR_INVALID_LDT_OFFSET = 563,
    ERROR_INVALID_LDT_DESCRIPTOR = 564,
    ERROR_TOO_MANY_THREADS = 565,
    ERROR_THREAD_NOT_IN_PROCESS = 566,
    ERROR_PAGEFILE_QUOTA_EXCEEDED = 567,
    ERROR_LOGON_SERVER_CONFLICT = 568,
    ERROR_SYNCHRONIZATION_REQUIRED = 569,
    ERROR_NET_OPEN_FAILED = 570,
    ERROR_IO_PRIVILEGE_FAILED = 571,
    ERROR_CONTROL_C_EXIT = 572,
    ERROR_MISSING_SYSTEMFILE = 573,
    ERROR_UNHANDLED_EXCEPTION = 574,
    ERROR_APP_INIT_FAILURE = 575,
    ERROR_PAGEFILE_CREATE_FAILED = 576,
    ERROR_INVALID_IMAGE_HASH = 577,
    ERROR_NO_PAGEFILE = 578,
    ERROR_ILLEGAL_FLOAT_CONTEXT = 579,
    ERROR_NO_EVENT_PAIR = 580,
    ERROR_DOMAIN_CTRLR_CONFIG_ERROR = 581,
    ERROR_ILLEGAL_CHARACTER = 582,
    ERROR_UNDEFINED_CHARACTER = 583,
    ERROR_FLOPPY_VOLUME = 584,
    ERROR_BIOS_FAILED_TO_CONNECT_INTERRUPT = 585,
    ERROR_BACKUP_CONTROLLER = 586,
    ERROR_MUTANT_LIMIT_EXCEEDED = 587,
    ERROR_FS_DRIVER_REQUIRED = 588,
    ERROR_CANNOT_LOAD_REGISTRY_FILE = 589,
    ERROR_DEBUG_ATTACH_FAILED = 590,
    ERROR_SYSTEM_PROCESS_TERMINATED = 591,
    ERROR_DATA_NOT_ACCEPTED = 592,
    ERROR_VDM_HARD_ERROR = 593,
    ERROR_DRIVER_CANCEL_TIMEOUT = 594,
    ERROR_REPLY_MESSAGE_MISMATCH = 595,
    ERROR_LOST_WRITEBEHIND_DATA = 596,
    ERROR_CLIENT_SERVER_PARAMETERS_INVALID = 597,
    ERROR_NOT_TINY_STREAM = 598,
    ERROR_STACK_OVERFLOW_READ = 599,
    ERROR_CONVERT_TO_LARGE = 600,
    ERROR_FOUND_OUT_OF_SCOPE = 601,
    ERROR_ALLOCATE_BUCKET = 602,
    ERROR_MARSHALL_OVERFLOW = 603,
    ERROR_INVALID_VARIANT = 604,
    ERROR_BAD_COMPRESSION_BUFFER = 605,
    ERROR_AUDIT_FAILED = 606,
    ERROR_TIMER_RESOLUTION_NOT_SET = 607,
    ERROR_INSUFFICIENT_LOGON_INFO = 608,
    ERROR_BAD_DLL_ENTRYPOINT = 609,
    ERROR_BAD_SERVICE_ENTRYPOINT = 610,
    ERROR_IP_ADDRESS_CONFLICT1 = 611,
    ERROR_IP_ADDRESS_CONFLICT2 = 612,
    ERROR_REGISTRY_QUOTA_LIMIT = 613,
    ERROR_NO_CALLBACK_ACTIVE = 614,
    ERROR_PWD_TOO_SHORT = 615,
    ERROR_PWD_TOO_RECENT = 616,
    ERROR_PWD_HISTORY_CONFLICT = 617,
    ERROR_UNSUPPORTED_COMPRESSION = 618,
    ERROR_INVALID_HW_PROFILE = 619,
    ERROR_INVALID_PLUGPLAY_DEVICE_PATH = 620,
    ERROR_QUOTA_LIST_INCONSISTENT = 621,
    ERROR_EVALUATION_EXPIRATION = 622,
    ERROR_ILLEGAL_DLL_RELOCATION = 623,
    ERROR_DLL_INIT_FAILED_LOGOFF = 624,
    ERROR_VALIDATE_CONTINUE = 625,
    ERROR_NO_MORE_MATCHES = 626,
    ERROR_RANGE_LIST_CONFLICT = 627,
    ERROR_SERVER_SID_MISMATCH = 628,
    ERROR_CANT_ENABLE_DENY_ONLY = 629,
    ERROR_FLOAT_MULTIPLE_FAULTS = 630,
    ERROR_FLOAT_MULTIPLE_TRAPS = 631,
    ERROR_NOINTERFACE = 632,
    ERROR_DRIVER_FAILED_SLEEP = 633,
    ERROR_CORRUPT_SYSTEM_FILE = 634,
    ERROR_COMMITMENT_MINIMUM = 635,
    ERROR_PNP_RESTART_ENUMERATION = 636,
    ERROR_SYSTEM_IMAGE_BAD_SIGNATURE = 637,
    ERROR_PNP_REBOOT_REQUIRED = 638,
    ERROR_INSUFFICIENT_POWER = 639,
    ERROR_MULTIPLE_FAULT_VIOLATION = 640,
    ERROR_SYSTEM_SHUTDOWN = 641,
    ERROR_PORT_NOT_SET = 642,
    ERROR_DS_VERSION_CHECK_FAILURE = 643,
    ERROR_RANGE_NOT_FOUND = 644,
    ERROR_NOT_SAFE_MODE_DRIVER = 646,
    ERROR_FAILED_DRIVER_ENTRY = 647,
    ERROR_DEVICE_ENUMERATION_ERROR = 648,
    ERROR_MOUNT_POINT_NOT_RESOLVED = 649,
    ERROR_INVALID_DEVICE_OBJECT_PARAMETER = 650,
    ERROR_MCA_OCCURED = 651,
    ERROR_DRIVER_DATABASE_ERROR = 652,
    ERROR_SYSTEM_HIVE_TOO_LARGE = 653,
    ERROR_DRIVER_FAILED_PRIOR_UNLOAD = 654,
    ERROR_VOLSNAP_PREPARE_HIBERNATE = 655,
    ERROR_HIBERNATION_FAILURE = 656,
    ERROR_PWD_TOO_LONG = 657,
    ERROR_FILE_SYSTEM_LIMITATION = 665,
    ERROR_ASSERTION_FAILURE = 668,
    ERROR_ACPI_ERROR = 669,
    ERROR_WOW_ASSERTION = 670,
    ERROR_PNP_BAD_MPS_TABLE = 671,
    ERROR_PNP_TRANSLATION_FAILED = 672,
    ERROR_PNP_IRQ_TRANSLATION_FAILED = 673,
    ERROR_PNP_INVALID_ID = 674,
    ERROR_WAKE_SYSTEM_DEBUGGER = 675,
    ERROR_HANDLES_CLOSED = 676,
    ERROR_EXTRANEOUS_INFORMATION = 677,
    ERROR_RXACT_COMMIT_NECESSARY = 678,
    ERROR_MEDIA_CHECK = 679,
    ERROR_GUID_SUBSTITUTION_MADE = 680,
    ERROR_STOPPED_ON_SYMLINK = 681,
    ERROR_LONGJUMP = 682,
    ERROR_PLUGPLAY_QUERY_VETOED = 683,
    ERROR_UNWIND_CONSOLIDATE = 684,
    ERROR_REGISTRY_HIVE_RECOVERED = 685,
    ERROR_DLL_MIGHT_BE_INSECURE = 686,
    ERROR_DLL_MIGHT_BE_INCOMPATIBLE = 687,
    ERROR_DBG_EXCEPTION_NOT_HANDLED = 688,
    ERROR_DBG_REPLY_LATER = 689,
    ERROR_DBG_UNABLE_TO_PROVIDE_HANDLE = 690,
    ERROR_DBG_TERMINATE_THREAD = 691,
    ERROR_DBG_TERMINATE_PROCESS = 692,
    ERROR_DBG_CONTROL_C = 693,
    ERROR_DBG_PRINTEXCEPTION_C = 694,
    ERROR_DBG_RIPEXCEPTION = 695,
    ERROR_DBG_CONTROL_BREAK = 696,
    ERROR_DBG_COMMAND_EXCEPTION = 697,
    ERROR_OBJECT_NAME_EXISTS = 698,
    ERROR_THREAD_WAS_SUSPENDED = 699,
    ERROR_IMAGE_NOT_AT_BASE = 700,
    ERROR_RXACT_STATE_CREATED = 701,
    ERROR_SEGMENT_NOTIFICATION = 702,
    ERROR_BAD_CURRENT_DIRECTORY = 703,
    ERROR_FT_READ_RECOVERY_FROM_BACKUP = 704,
    ERROR_FT_WRITE_RECOVERY = 705,
    ERROR_IMAGE_MACHINE_TYPE_MISMATCH = 706,
    ERROR_RECEIVE_PARTIAL = 707,
    ERROR_RECEIVE_EXPEDITED = 708,
    ERROR_RECEIVE_PARTIAL_EXPEDITED = 709,
    ERROR_EVENT_DONE = 710,
    ERROR_EVENT_PENDING = 711,
    ERROR_CHECKING_FILE_SYSTEM = 712,
    ERROR_FATAL_APP_EXIT = 713,
    ERROR_PREDEFINED_HANDLE = 714,
    ERROR_WAS_UNLOCKED = 715,
    ERROR_SERVICE_NOTIFICATION = 716,
    ERROR_WAS_LOCKED = 717,
    ERROR_LOG_HARD_ERROR = 718,
    ERROR_ALREADY_WIN32 = 719,
    ERROR_IMAGE_MACHINE_TYPE_MISMATCH_EXE = 720,
    ERROR_NO_YIELD_PERFORMED = 721,
    ERROR_TIMER_RESUME_IGNORED = 722,
    ERROR_ARBITRATION_UNHANDLED = 723,
    ERROR_CARDBUS_NOT_SUPPORTED = 724,
    ERROR_MP_PROCESSOR_MISMATCH = 725,
    ERROR_HIBERNATED = 726,
    ERROR_RESUME_HIBERNATION = 727,
    ERROR_FIRMWARE_UPDATED = 728,
    ERROR_DRIVERS_LEAKING_LOCKED_PAGES = 729,
    ERROR_WAKE_SYSTEM = 730,
    ERROR_WAIT_1 = 731,
    ERROR_WAIT_2 = 732,
    ERROR_WAIT_3 = 733,
    ERROR_WAIT_63 = 734,
    ERROR_ABANDONED_WAIT_0 = 735,
    ERROR_ABANDONED_WAIT_63 = 736,
    ERROR_USER_APC = 737,
    ERROR_KERNEL_APC = 738,
    ERROR_ALERTED = 739,
    ERROR_ELEVATION_REQUIRED = 740,
    ERROR_REPARSE = 741,
    ERROR_OPLOCK_BREAK_IN_PROGRESS = 742,
    ERROR_VOLUME_MOUNTED = 743,
    ERROR_RXACT_COMMITTED = 744,
    ERROR_NOTIFY_CLEANUP = 745,
    ERROR_PRIMARY_TRANSPORT_CONNECT_FAILED = 746,
    ERROR_PAGE_FAULT_TRANSITION = 747,
    ERROR_PAGE_FAULT_DEMAND_ZERO = 748,
    ERROR_PAGE_FAULT_COPY_ON_WRITE = 749,
    ERROR_PAGE_FAULT_GUARD_PAGE = 750,
    ERROR_PAGE_FAULT_PAGING_FILE = 751,
    ERROR_CACHE_PAGE_LOCKED = 752,
    ERROR_CRASH_DUMP = 753,
    ERROR_BUFFER_ALL_ZEROS = 754,
    ERROR_REPARSE_OBJECT = 755,
    ERROR_RESOURCE_REQUIREMENTS_CHANGED = 756,
    ERROR_TRANSLATION_COMPLETE = 757,
    ERROR_NOTHING_TO_TERMINATE = 758,
    ERROR_PROCESS_NOT_IN_JOB = 759,
    ERROR_PROCESS_IN_JOB = 760,
    ERROR_VOLSNAP_HIBERNATE_READY = 761,
    ERROR_FSFILTER_OP_COMPLETED_SUCCESSFULLY = 762,
    ERROR_INTERRUPT_VECTOR_ALREADY_CONNECTED = 763,
    ERROR_INTERRUPT_STILL_CONNECTED = 764,
    ERROR_WAIT_FOR_OPLOCK = 765,
    ERROR_DBG_EXCEPTION_HANDLED = 766,
    ERROR_DBG_CONTINUE = 767,
    ERROR_CALLBACK_POP_STACK = 768,
    ERROR_COMPRESSION_DISABLED = 769,
    ERROR_CANTFETCHBACKWARDS = 770,
    ERROR_CANTSCROLLBACKWARDS = 771,
    ERROR_ROWSNOTRELEASED = 772,
    ERROR_BAD_ACCESSOR_FLAGS = 773,
    ERROR_ERRORS_ENCOUNTERED = 774,
    ERROR_NOT_CAPABLE = 775,
    ERROR_REQUEST_OUT_OF_SEQUENCE = 776,
    ERROR_VERSION_PARSE_ERROR = 777,
    ERROR_BADSTARTPOSITION = 778,
    ERROR_MEMORY_HARDWARE = 779,
    ERROR_DISK_REPAIR_DISABLED = 780,
    ERROR_INSUFFICIENT_RESOURCE_FOR_SPECIFIED_SHARED_SECTION_SIZE = 781,
    ERROR_SYSTEM_POWERSTATE_TRANSITION = 782,
    ERROR_SYSTEM_POWERSTATE_COMPLEX_TRANSITION = 783,
    ERROR_MCA_EXCEPTION = 784,
    ERROR_ACCESS_AUDIT_BY_POLICY = 785,
    ERROR_ACCESS_DISABLED_NO_SAFER_UI_BY_POLICY = 786,
    ERROR_ABANDON_HIBERFILE = 787,
    ERROR_LOST_WRITEBEHIND_DATA_NETWORK_DISCONNECTED = 788,
    ERROR_LOST_WRITEBEHIND_DATA_NETWORK_SERVER_ERROR = 789,
    ERROR_LOST_WRITEBEHIND_DATA_LOCAL_DISK_ERROR = 790,
    ERROR_BAD_MCFG_TABLE = 791,
    ERROR_DISK_REPAIR_REDIRECTED = 792,
    ERROR_DISK_REPAIR_UNSUCCESSFUL = 793,
    ERROR_CORRUPT_LOG_OVERFULL = 794,
    ERROR_CORRUPT_LOG_CORRUPTED = 795,
    ERROR_CORRUPT_LOG_UNAVAILABLE = 796,
    ERROR_CORRUPT_LOG_DELETED_FULL = 797,
    ERROR_CORRUPT_LOG_CLEARED = 798,
    ERROR_ORPHAN_NAME_EXHAUSTED = 799,
    ERROR_OPLOCK_SWITCHED_TO_NEW_HANDLE = 800,
    ERROR_CANNOT_GRANT_REQUESTED_OPLOCK = 801,
    ERROR_CANNOT_BREAK_OPLOCK = 802,
    ERROR_OPLOCK_HANDLE_CLOSED = 803,
    ERROR_NO_ACE_CONDITION = 804,
    ERROR_INVALID_ACE_CONDITION = 805,
    ERROR_FILE_HANDLE_REVOKED = 806,
    ERROR_IMAGE_AT_DIFFERENT_BASE = 807,
    ERROR_ENCRYPTED_IO_NOT_POSSIBLE = 808,
    ERROR_FILE_METADATA_OPTIMIZATION_IN_PROGRESS = 809,
    ERROR_QUOTA_ACTIVITY = 810,
    ERROR_HANDLE_REVOKED = 811,
    ERROR_CALLBACK_INVOKE_INLINE = 812,
    ERROR_CPU_SET_INVALID = 813,
    ERROR_ENCLAVE_NOT_TERMINATED = 814,
    ERROR_ENCLAVE_VIOLATION = 815,
    ERROR_EA_ACCESS_DENIED = 994,
    ERROR_OPERATION_ABORTED = 995,
    ERROR_IO_INCOMPLETE = 996,
    ERROR_IO_PENDING = 997,
    ERROR_NOACCESS = 998,
    ERROR_SWAPERROR = 999,
    ERROR_STACK_OVERFLOW = 1001,
    ERROR_INVALID_MESSAGE = 1002,
    ERROR_CAN_NOT_COMPLETE = 1003,
    ERROR_INVALID_FLAGS = 1004,
    ERROR_UNRECOGNIZED_VOLUME = 1005,
    ERROR_FILE_INVALID = 1006,
    ERROR_FULLSCREEN_MODE = 1007,
    ERROR_NO_TOKEN = 1008,
    ERROR_BADDB = 1009,
    ERROR_BADKEY = 1010,
    ERROR_CANTOPEN = 1011,
    ERROR_CANTREAD = 1012,
    ERROR_CANTWRITE = 1013,
    ERROR_REGISTRY_RECOVERED = 1014,
    ERROR_REGISTRY_CORRUPT = 1015,
    ERROR_REGISTRY_IO_FAILED = 1016,
    ERROR_NOT_REGISTRY_FILE = 1017,
    ERROR_KEY_DELETED = 1018,
    ERROR_NO_LOG_SPACE = 1019,
    ERROR_KEY_HAS_CHILDREN = 1020,
    ERROR_CHILD_MUST_BE_VOLATILE = 1021,
    ERROR_NOTIFY_ENUM_DIR = 1022,
    ERROR_DEPENDENT_SERVICES_RUNNING = 1051,
    ERROR_INVALID_SERVICE_CONTROL = 1052,
    ERROR_SERVICE_REQUEST_TIMEOUT = 1053,
    ERROR_SERVICE_NO_THREAD = 1054,
    ERROR_SERVICE_DATABASE_LOCKED = 1055,
    ERROR_SERVICE_ALREADY_RUNNING = 1056,
    ERROR_INVALID_SERVICE_ACCOUNT = 1057,
    ERROR_SERVICE_DISABLED = 1058,
    ERROR_CIRCULAR_DEPENDENCY = 1059,
    ERROR_SERVICE_DOES_NOT_EXIST = 1060,
    ERROR_SERVICE_CANNOT_ACCEPT_CTRL = 1061,
    ERROR_SERVICE_NOT_ACTIVE = 1062,
    ERROR_FAILED_SERVICE_CONTROLLER_CONNECT = 1063,
    ERROR_EXCEPTION_IN_SERVICE = 1064,
    ERROR_DATABASE_DOES_NOT_EXIST = 1065,
    ERROR_SERVICE_SPECIFIC_ERROR = 1066,
    ERROR_PROCESS_ABORTED = 1067,
    ERROR_SERVICE_DEPENDENCY_FAIL = 1068,
    ERROR_SERVICE_LOGON_FAILED = 1069,
    ERROR_SERVICE_START_HANG = 1070,
    ERROR_INVALID_SERVICE_LOCK = 1071,
    ERROR_SERVICE_MARKED_FOR_DELETE = 1072,
    ERROR_SERVICE_EXISTS = 1073,
    ERROR_ALREADY_RUNNING_LKG = 1074,
    ERROR_SERVICE_DEPENDENCY_DELETED = 1075,
    ERROR_BOOT_ALREADY_ACCEPTED = 1076,
    ERROR_SERVICE_NEVER_STARTED = 1077,
    ERROR_DUPLICATE_SERVICE_NAME = 1078,
    ERROR_DIFFERENT_SERVICE_ACCOUNT = 1079,
    ERROR_CANNOT_DETECT_DRIVER_FAILURE = 1080,
    ERROR_CANNOT_DETECT_PROCESS_ABORT = 1081,
    ERROR_NO_RECOVERY_PROGRAM = 1082,
    ERROR_SERVICE_NOT_IN_EXE = 1083,
    ERROR_NOT_SAFEBOOT_SERVICE = 1084,
    ERROR_END_OF_MEDIA = 1100,
    ERROR_FILEMARK_DETECTED = 1101,
    ERROR_BEGINNING_OF_MEDIA = 1102,
    ERROR_SETMARK_DETECTED = 1103,
    ERROR_NO_DATA_DETECTED = 1104,
    ERROR_PARTITION_FAILURE = 1105,
    ERROR_INVALID_BLOCK_LENGTH = 1106,
    ERROR_DEVICE_NOT_PARTITIONED = 1107,
    ERROR_UNABLE_TO_LOCK_MEDIA = 1108,
    ERROR_UNABLE_TO_UNLOAD_MEDIA = 1109,
    ERROR_MEDIA_CHANGED = 1110,
    ERROR_BUS_RESET = 1111,
    ERROR_NO_MEDIA_IN_DRIVE = 1112,
    ERROR_NO_UNICODE_TRANSLATION = 1113,
    ERROR_DLL_INIT_FAILED = 1114,
    ERROR_SHUTDOWN_IN_PROGRESS = 1115,
    ERROR_NO_SHUTDOWN_IN_PROGRESS = 1116,
    ERROR_IO_DEVICE = 1117,
    ERROR_SERIAL_NO_DEVICE = 1118,
    ERROR_IRQ_BUSY = 1119,
    ERROR_MORE_WRITES = 1120,
    ERROR_COUNTER_TIMEOUT = 1121,
    ERROR_FLOPPY_ID_MARK_NOT_FOUND = 1122,
    ERROR_FLOPPY_WRONG_CYLINDER = 1123,
    ERROR_FLOPPY_UNKNOWN_ERROR = 1124,
    ERROR_FLOPPY_BAD_REGISTERS = 1125,
    ERROR_DISK_RECALIBRATE_FAILED = 1126,
    ERROR_DISK_OPERATION_FAILED = 1127,
    ERROR_DISK_RESET_FAILED = 1128,
    ERROR_EOM_OVERFLOW = 1129,
    ERROR_NOT_ENOUGH_SERVER_MEMORY = 1130,
    ERROR_POSSIBLE_DEADLOCK = 1131,
    ERROR_MAPPED_ALIGNMENT = 1132,
    ERROR_SET_POWER_STATE_VETOED = 1140,
    ERROR_SET_POWER_STATE_FAILED = 1141,
    ERROR_TOO_MANY_LINKS = 1142,
    ERROR_OLD_WIN_VERSION = 1150,
    ERROR_APP_WRONG_OS = 1151,
    ERROR_SINGLE_INSTANCE_APP = 1152,
    ERROR_RMODE_APP = 1153,
    ERROR_INVALID_DLL = 1154,
    ERROR_NO_ASSOCIATION = 1155,
    ERROR_DDE_FAIL = 1156,
    ERROR_DLL_NOT_FOUND = 1157,
    ERROR_NO_MORE_USER_HANDLES = 1158,
    ERROR_MESSAGE_SYNC_ONLY = 1159,
    ERROR_SOURCE_ELEMENT_EMPTY = 1160,
    ERROR_DESTINATION_ELEMENT_FULL = 1161,
    ERROR_ILLEGAL_ELEMENT_ADDRESS = 1162,
    ERROR_MAGAZINE_NOT_PRESENT = 1163,
    ERROR_DEVICE_REINITIALIZATION_NEEDED = 1164,
    ERROR_DEVICE_REQUIRES_CLEANING = 1165,
    ERROR_DEVICE_DOOR_OPEN = 1166,
    ERROR_DEVICE_NOT_CONNECTED = 1167,
    ERROR_NOT_FOUND = 1168,
    ERROR_NO_MATCH = 1169,
    ERROR_SET_NOT_FOUND = 1170,
    ERROR_POINT_NOT_FOUND = 1171,
    ERROR_NO_TRACKING_SERVICE = 1172,
    ERROR_NO_VOLUME_ID = 1173,
    ERROR_UNABLE_TO_REMOVE_REPLACED = 1175,
    ERROR_UNABLE_TO_MOVE_REPLACEMENT = 1176,
    ERROR_UNABLE_TO_MOVE_REPLACEMENT_2 = 1177,
    ERROR_JOURNAL_DELETE_IN_PROGRESS = 1178,
    ERROR_JOURNAL_NOT_ACTIVE = 1179,
    ERROR_POTENTIAL_FILE_FOUND = 1180,
    ERROR_JOURNAL_ENTRY_DELETED = 1181,
    ERROR_VRF_CFG_AND_IO_ENABLED = 1183,
    ERROR_PARTITION_TERMINATING = 1184,
    ERROR_SHUTDOWN_IS_SCHEDULED = 1190,
    ERROR_SHUTDOWN_USERS_LOGGED_ON = 1191,
    ERROR_BAD_DEVICE = 1200,
    ERROR_CONNECTION_UNAVAIL = 1201,
    ERROR_DEVICE_ALREADY_REMEMBERED = 1202,
    ERROR_NO_NET_OR_BAD_PATH = 1203,
    ERROR_BAD_PROVIDER = 1204,
    ERROR_CANNOT_OPEN_PROFILE = 1205,
    ERROR_BAD_PROFILE = 1206,
    ERROR_NOT_CONTAINER = 1207,
    ERROR_EXTENDED_ERROR = 1208,
    ERROR_INVALID_GROUPNAME = 1209,
    ERROR_INVALID_COMPUTERNAME = 1210,
    ERROR_INVALID_EVENTNAME = 1211,
    ERROR_INVALID_DOMAINNAME = 1212,
    ERROR_INVALID_SERVICENAME = 1213,
    ERROR_INVALID_NETNAME = 1214,
    ERROR_INVALID_SHARENAME = 1215,
    ERROR_INVALID_PASSWORDNAME = 1216,
    ERROR_INVALID_MESSAGENAME = 1217,
    ERROR_INVALID_MESSAGEDEST = 1218,
    ERROR_SESSION_CREDENTIAL_CONFLICT = 1219,
    ERROR_REMOTE_SESSION_LIMIT_EXCEEDED = 1220,
    ERROR_DUP_DOMAINNAME = 1221,
    ERROR_NO_NETWORK = 1222,
    ERROR_CANCELLED = 1223,
    ERROR_USER_MAPPED_FILE = 1224,
    ERROR_CONNECTION_REFUSED = 1225,
    ERROR_GRACEFUL_DISCONNECT = 1226,
    ERROR_ADDRESS_ALREADY_ASSOCIATED = 1227,
    ERROR_ADDRESS_NOT_ASSOCIATED = 1228,
    ERROR_CONNECTION_INVALID = 1229,
    ERROR_CONNECTION_ACTIVE = 1230,
    ERROR_NETWORK_UNREACHABLE = 1231,
    ERROR_HOST_UNREACHABLE = 1232,
    ERROR_PROTOCOL_UNREACHABLE = 1233,
    ERROR_PORT_UNREACHABLE = 1234,
    ERROR_REQUEST_ABORTED = 1235,
    ERROR_CONNECTION_ABORTED = 1236,
    ERROR_RETRY = 1237,
    ERROR_CONNECTION_COUNT_LIMIT = 1238,
    ERROR_LOGIN_TIME_RESTRICTION = 1239,
    ERROR_LOGIN_WKSTA_RESTRICTION = 1240,
    ERROR_INCORRECT_ADDRESS = 1241,
    ERROR_ALREADY_REGISTERED = 1242,
    ERROR_SERVICE_NOT_FOUND = 1243,
    ERROR_NOT_AUTHENTICATED = 1244,
    ERROR_NOT_LOGGED_ON = 1245,
    ERROR_CONTINUE = 1246,
    ERROR_ALREADY_INITIALIZED = 1247,
    ERROR_NO_MORE_DEVICES = 1248,
    ERROR_NO_SUCH_SITE = 1249,
    ERROR_DOMAIN_CONTROLLER_EXISTS = 1250,
    ERROR_ONLY_IF_CONNECTED = 1251,
    ERROR_OVERRIDE_NOCHANGES = 1252,
    ERROR_BAD_USER_PROFILE = 1253,
    ERROR_NOT_SUPPORTED_ON_SBS = 1254,
    ERROR_SERVER_SHUTDOWN_IN_PROGRESS = 1255,
    ERROR_HOST_DOWN = 1256,
    ERROR_NON_ACCOUNT_SID = 1257,
    ERROR_NON_DOMAIN_SID = 1258,
    ERROR_APPHELP_BLOCK = 1259,
    ERROR_ACCESS_DISABLED_BY_POLICY = 1260,
    ERROR_REG_NAT_CONSUMPTION = 1261,
    ERROR_CSCSHARE_OFFLINE = 1262,
    ERROR_PKINIT_FAILURE = 1263,
    ERROR_SMARTCARD_SUBSYSTEM_FAILURE = 1264,
    ERROR_DOWNGRADE_DETECTED = 1265,
    ERROR_MACHINE_LOCKED = 1271,
    ERROR_SMB_GUEST_LOGON_BLOCKED = 1272,
    ERROR_CALLBACK_SUPPLIED_INVALID_DATA = 1273,
    ERROR_SYNC_FOREGROUND_REFRESH_REQUIRED = 1274,
    ERROR_DRIVER_BLOCKED = 1275,
    ERROR_INVALID_IMPORT_OF_NON_DLL = 1276,
    ERROR_ACCESS_DISABLED_WEBBLADE = 1277,
    ERROR_ACCESS_DISABLED_WEBBLADE_TAMPER = 1278,
    ERROR_RECOVERY_FAILURE = 1279,
    ERROR_ALREADY_FIBER = 1280,
    ERROR_ALREADY_THREAD = 1281,
    ERROR_STACK_BUFFER_OVERRUN = 1282,
    ERROR_PARAMETER_QUOTA_EXCEEDED = 1283,
    ERROR_DEBUGGER_INACTIVE = 1284,
    ERROR_DELAY_LOAD_FAILED = 1285,
    ERROR_VDM_DISALLOWED = 1286,
    ERROR_UNIDENTIFIED_ERROR = 1287,
    ERROR_INVALID_CRUNTIME_PARAMETER = 1288,
    ERROR_BEYOND_VDL = 1289,
    ERROR_INCOMPATIBLE_SERVICE_SID_TYPE = 1290,
    ERROR_DRIVER_PROCESS_TERMINATED = 1291,
    ERROR_IMPLEMENTATION_LIMIT = 1292,
    ERROR_PROCESS_IS_PROTECTED = 1293,
    ERROR_SERVICE_NOTIFY_CLIENT_LAGGING = 1294,
    ERROR_DISK_QUOTA_EXCEEDED = 1295,
    ERROR_CONTENT_BLOCKED = 1296,
    ERROR_INCOMPATIBLE_SERVICE_PRIVILEGE = 1297,
    ERROR_APP_HANG = 1298,
    ERROR_INVALID_LABEL = 1299,
    ERROR_NOT_ALL_ASSIGNED = 1300,
    ERROR_SOME_NOT_MAPPED = 1301,
    ERROR_NO_QUOTAS_FOR_ACCOUNT = 1302,
    ERROR_LOCAL_USER_SESSION_KEY = 1303,
    ERROR_NULL_LM_PASSWORD = 1304,
    ERROR_UNKNOWN_REVISION = 1305,
    ERROR_REVISION_MISMATCH = 1306,
    ERROR_INVALID_OWNER = 1307,
    ERROR_INVALID_PRIMARY_GROUP = 1308,
    ERROR_NO_IMPERSONATION_TOKEN = 1309,
    ERROR_CANT_DISABLE_MANDATORY = 1310,
    ERROR_NO_LOGON_SERVERS = 1311,
    ERROR_NO_SUCH_LOGON_SESSION = 1312,
    ERROR_NO_SUCH_PRIVILEGE = 1313,
    ERROR_PRIVILEGE_NOT_HELD = 1314,
    ERROR_INVALID_ACCOUNT_NAME = 1315,
    ERROR_USER_EXISTS = 1316,
    ERROR_NO_SUCH_USER = 1317,
    ERROR_GROUP_EXISTS = 1318,
    ERROR_NO_SUCH_GROUP = 1319,
    ERROR_MEMBER_IN_GROUP = 1320,
    ERROR_MEMBER_NOT_IN_GROUP = 1321,
    ERROR_LAST_ADMIN = 1322,
    ERROR_WRONG_PASSWORD = 1323,
    ERROR_ILL_FORMED_PASSWORD = 1324,
    ERROR_PASSWORD_RESTRICTION = 1325,
    ERROR_LOGON_FAILURE = 1326,
    ERROR_ACCOUNT_RESTRICTION = 1327,
    ERROR_INVALID_LOGON_HOURS = 1328,
    ERROR_INVALID_WORKSTATION = 1329,
    ERROR_PASSWORD_EXPIRED = 1330,
    ERROR_ACCOUNT_DISABLED = 1331,
    ERROR_NONE_MAPPED = 1332,
    ERROR_TOO_MANY_LUIDS_REQUESTED = 1333,
    ERROR_LUIDS_EXHAUSTED = 1334,
    ERROR_INVALID_SUB_AUTHORITY = 1335,
    ERROR_INVALID_ACL = 1336,
    ERROR_INVALID_SID = 1337,
    ERROR_INVALID_SECURITY_DESCR = 1338,
    ERROR_BAD_INHERITANCE_ACL = 1340,
    ERROR_SERVER_DISABLED = 1341,
    ERROR_SERVER_NOT_DISABLED = 1342,
    ERROR_INVALID_ID_AUTHORITY = 1343,
    ERROR_ALLOTTED_SPACE_EXCEEDED = 1344,
    ERROR_INVALID_GROUP_ATTRIBUTES = 1345,
    ERROR_BAD_IMPERSONATION_LEVEL = 1346,
    ERROR_CANT_OPEN_ANONYMOUS = 1347,
    ERROR_BAD_VALIDATION_CLASS = 1348,
    ERROR_BAD_TOKEN_TYPE = 1349,
    ERROR_NO_SECURITY_ON_OBJECT = 1350,
    ERROR_CANT_ACCESS_DOMAIN_INFO = 1351,
    ERROR_INVALID_SERVER_STATE = 1352,
    ERROR_INVALID_DOMAIN_STATE = 1353,
    ERROR_INVALID_DOMAIN_ROLE = 1354,
    ERROR_NO_SUCH_DOMAIN = 1355,
    ERROR_DOMAIN_EXISTS = 1356,
    ERROR_DOMAIN_LIMIT_EXCEEDED = 1357,
    ERROR_INTERNAL_DB_CORRUPTION = 1358,
    ERROR_INTERNAL_ERROR = 1359,
    ERROR_GENERIC_NOT_MAPPED = 1360,
    ERROR_BAD_DESCRIPTOR_FORMAT = 1361,
    ERROR_NOT_LOGON_PROCESS = 1362,
    ERROR_LOGON_SESSION_EXISTS = 1363,
    ERROR_NO_SUCH_PACKAGE = 1364,
    ERROR_BAD_LOGON_SESSION_STATE = 1365,
    ERROR_LOGON_SESSION_COLLISION = 1366,
    ERROR_INVALID_LOGON_TYPE = 1367,
    ERROR_CANNOT_IMPERSONATE = 1368,
    ERROR_RXACT_INVALID_STATE = 1369,
    ERROR_RXACT_COMMIT_FAILURE = 1370,
    ERROR_SPECIAL_ACCOUNT = 1371,
    ERROR_SPECIAL_GROUP = 1372,
    ERROR_SPECIAL_USER = 1373,
    ERROR_MEMBERS_PRIMARY_GROUP = 1374,
    ERROR_TOKEN_ALREADY_IN_USE = 1375,
    ERROR_NO_SUCH_ALIAS = 1376,
    ERROR_MEMBER_NOT_IN_ALIAS = 1377,
    ERROR_MEMBER_IN_ALIAS = 1378,
    ERROR_ALIAS_EXISTS = 1379,
    ERROR_LOGON_NOT_GRANTED = 1380,
    ERROR_TOO_MANY_SECRETS = 1381,
    ERROR_SECRET_TOO_LONG = 1382,
    ERROR_INTERNAL_DB_ERROR = 1383,
    ERROR_TOO_MANY_CONTEXT_IDS = 1384,
    ERROR_LOGON_TYPE_NOT_GRANTED = 1385,
    ERROR_NT_CROSS_ENCRYPTION_REQUIRED = 1386,
    ERROR_NO_SUCH_MEMBER = 1387,
    ERROR_INVALID_MEMBER = 1388,
    ERROR_TOO_MANY_SIDS = 1389,
    ERROR_LM_CROSS_ENCRYPTION_REQUIRED = 1390,
    ERROR_NO_INHERITANCE = 1391,
    ERROR_FILE_CORRUPT = 1392,
    ERROR_DISK_CORRUPT = 1393,
    ERROR_NO_USER_SESSION_KEY = 1394,
    ERROR_LICENSE_QUOTA_EXCEEDED = 1395,
    ERROR_WRONG_TARGET_NAME = 1396,
    ERROR_MUTUAL_AUTH_FAILED = 1397,
    ERROR_TIME_SKEW = 1398,
    ERROR_CURRENT_DOMAIN_NOT_ALLOWED = 1399,
    ERROR_INVALID_WINDOW_HANDLE = 1400,
    ERROR_INVALID_MENU_HANDLE = 1401,
    ERROR_INVALID_CURSOR_HANDLE = 1402,
    ERROR_INVALID_ACCEL_HANDLE = 1403,
    ERROR_INVALID_HOOK_HANDLE = 1404,
    ERROR_INVALID_DWP_HANDLE = 1405,
    ERROR_TLW_WITH_WSCHILD = 1406,
    ERROR_CANNOT_FIND_WND_CLASS = 1407,
    ERROR_WINDOW_OF_OTHER_THREAD = 1408,
    ERROR_HOTKEY_ALREADY_REGISTERED = 1409,
    ERROR_CLASS_ALREADY_EXISTS = 1410,
    ERROR_CLASS_DOES_NOT_EXIST = 1411,
    ERROR_CLASS_HAS_WINDOWS = 1412,
    ERROR_INVALID_INDEX = 1413,
    ERROR_INVALID_ICON_HANDLE = 1414,
    ERROR_PRIVATE_DIALOG_INDEX = 1415,
    ERROR_LISTBOX_ID_NOT_FOUND = 1416,
    ERROR_NO_WILDCARD_CHARACTERS = 1417,
    ERROR_CLIPBOARD_NOT_OPEN = 1418,
    ERROR_HOTKEY_NOT_REGISTERED = 1419,
    ERROR_WINDOW_NOT_DIALOG = 1420,
    ERROR_CONTROL_ID_NOT_FOUND = 1421,
    ERROR_INVALID_COMBOBOX_MESSAGE = 1422,
    ERROR_WINDOW_NOT_COMBOBOX = 1423,
    ERROR_INVALID_EDIT_HEIGHT = 1424,
    ERROR_DC_NOT_FOUND = 1425,
    ERROR_INVALID_HOOK_FILTER = 1426,
    ERROR_INVALID_FILTER_PROC = 1427,
    ERROR_HOOK_NEEDS_HMOD = 1428,
    ERROR_GLOBAL_ONLY_HOOK = 1429,
    ERROR_JOURNAL_HOOK_SET = 1430,
    ERROR_HOOK_NOT_INSTALLED = 1431,
    ERROR_INVALID_LB_MESSAGE = 1432,
    ERROR_SETCOUNT_ON_BAD_LB = 1433,
    ERROR_LB_WITHOUT_TABSTOPS = 1434,
    ERROR_DESTROY_OBJECT_OF_OTHER_THREAD = 1435,
    ERROR_CHILD_WINDOW_MENU = 1436,
    ERROR_NO_SYSTEM_MENU = 1437,
    ERROR_INVALID_MSGBOX_STYLE = 1438,
    ERROR_INVALID_SPI_VALUE = 1439,
    ERROR_SCREEN_ALREADY_LOCKED = 1440,
    ERROR_HWNDS_HAVE_DIFF_PARENT = 1441,
    ERROR_NOT_CHILD_WINDOW = 1442,
    ERROR_INVALID_GW_COMMAND = 1443,
    ERROR_INVALID_THREAD_ID = 1444,
    ERROR_NON_MDICHILD_WINDOW = 1445,
    ERROR_POPUP_ALREADY_ACTIVE = 1446,
    ERROR_NO_SCROLLBARS = 1447,
    ERROR_INVALID_SCROLLBAR_RANGE = 1448,
    ERROR_INVALID_SHOWWIN_COMMAND = 1449,
    ERROR_NO_SYSTEM_RESOURCES = 1450,
    ERROR_NONPAGED_SYSTEM_RESOURCES = 1451,
    ERROR_PAGED_SYSTEM_RESOURCES = 1452,
    ERROR_WORKING_SET_QUOTA = 1453,
    ERROR_PAGEFILE_QUOTA = 1454,
    ERROR_COMMITMENT_LIMIT = 1455,
    ERROR_MENU_ITEM_NOT_FOUND = 1456,
    ERROR_INVALID_KEYBOARD_HANDLE = 1457,
    ERROR_HOOK_TYPE_NOT_ALLOWED = 1458,
    ERROR_REQUIRES_INTERACTIVE_WINDOWSTATION = 1459,
    ERROR_TIMEOUT = 1460,
    ERROR_INVALID_MONITOR_HANDLE = 1461,
    ERROR_INCORRECT_SIZE = 1462,
    ERROR_SYMLINK_CLASS_DISABLED = 1463,
    ERROR_SYMLINK_NOT_SUPPORTED = 1464,
    ERROR_XML_PARSE_ERROR = 1465,
    ERROR_XMLDSIG_ERROR = 1466,
    ERROR_RESTART_APPLICATION = 1467,
    ERROR_WRONG_COMPARTMENT = 1468,
    ERROR_AUTHIP_FAILURE = 1469,
    ERROR_NO_NVRAM_RESOURCES = 1470,
    ERROR_NOT_GUI_PROCESS = 1471,
    ERROR_EVENTLOG_FILE_CORRUPT = 1500,
    ERROR_EVENTLOG_CANT_START = 1501,
    ERROR_LOG_FILE_FULL = 1502,
    ERROR_EVENTLOG_FILE_CHANGED = 1503,
    ERROR_CONTAINER_ASSIGNED = 1504,
    ERROR_JOB_NO_CONTAINER = 1505,
    ERROR_INVALID_TASK_NAME = 1550,
    ERROR_INVALID_TASK_INDEX = 1551,
    ERROR_THREAD_ALREADY_IN_TASK = 1552,
    ERROR_INSTALL_SERVICE_FAILURE = 1601,
    ERROR_INSTALL_USEREXIT = 1602,
    ERROR_INSTALL_FAILURE = 1603,
    ERROR_INSTALL_SUSPEND = 1604,
    ERROR_UNKNOWN_PRODUCT = 1605,
    ERROR_UNKNOWN_FEATURE = 1606,
    ERROR_UNKNOWN_COMPONENT = 1607,
    ERROR_UNKNOWN_PROPERTY = 1608,
    ERROR_INVALID_HANDLE_STATE = 1609,
    ERROR_BAD_CONFIGURATION = 1610,
    ERROR_INDEX_ABSENT = 1611,
    ERROR_INSTALL_SOURCE_ABSENT = 1612,
    ERROR_INSTALL_PACKAGE_VERSION = 1613,
    ERROR_PRODUCT_UNINSTALLED = 1614,
    ERROR_BAD_QUERY_SYNTAX = 1615,
    ERROR_INVALID_FIELD = 1616,
    ERROR_DEVICE_REMOVED = 1617,
    ERROR_INSTALL_ALREADY_RUNNING = 1618,
    ERROR_INSTALL_PACKAGE_OPEN_FAILED = 1619,
    ERROR_INSTALL_PACKAGE_INVALID = 1620,
    ERROR_INSTALL_UI_FAILURE = 1621,
    ERROR_INSTALL_LOG_FAILURE = 1622,
    ERROR_INSTALL_LANGUAGE_UNSUPPORTED = 1623,
    ERROR_INSTALL_TRANSFORM_FAILURE = 1624,
    ERROR_INSTALL_PACKAGE_REJECTED = 1625,
    ERROR_FUNCTION_NOT_CALLED = 1626,
    ERROR_FUNCTION_FAILED = 1627,
    ERROR_INVALID_TABLE = 1628,
    ERROR_DATATYPE_MISMATCH = 1629,
    ERROR_UNSUPPORTED_TYPE = 1630,
    ERROR_CREATE_FAILED = 1631,
    ERROR_INSTALL_TEMP_UNWRITABLE = 1632,
    ERROR_INSTALL_PLATFORM_UNSUPPORTED = 1633,
    ERROR_INSTALL_NOTUSED = 1634,
    ERROR_PATCH_PACKAGE_OPEN_FAILED = 1635,
    ERROR_PATCH_PACKAGE_INVALID = 1636,
    ERROR_PATCH_PACKAGE_UNSUPPORTED = 1637,
    ERROR_PRODUCT_VERSION = 1638,
    ERROR_INVALID_COMMAND_LINE = 1639,
    ERROR_INSTALL_REMOTE_DISALLOWED = 1640,
    ERROR_SUCCESS_REBOOT_INITIATED = 1641,
    ERROR_PATCH_TARGET_NOT_FOUND = 1642,
    ERROR_PATCH_PACKAGE_REJECTED = 1643,
    ERROR_INSTALL_TRANSFORM_REJECTED = 1644,
    ERROR_INSTALL_REMOTE_PROHIBITED = 1645,
    ERROR_PATCH_REMOVAL_UNSUPPORTED = 1646,
    ERROR_UNKNOWN_PATCH = 1647,
    ERROR_PATCH_NO_SEQUENCE = 1648,
    ERROR_PATCH_REMOVAL_DISALLOWED = 1649,
    ERROR_INVALID_PATCH_XML = 1650,
    ERROR_PATCH_MANAGED_ADVERTISED_PRODUCT = 1651,
    ERROR_INSTALL_SERVICE_SAFEBOOT = 1652,
    ERROR_FAIL_FAST_EXCEPTION = 1653,
    ERROR_INSTALL_REJECTED = 1654,
    ERROR_DYNAMIC_CODE_BLOCKED = 1655,
    ERROR_NOT_SAME_OBJECT = 1656,
    ERROR_STRICT_CFG_VIOLATION = 1657,
    ERROR_SET_CONTEXT_DENIED = 1660,
    ERROR_CROSS_PARTITION_VIOLATION = 1661,
    ERROR_RETURN_ADDRESS_HIJACK_ATTEMPT = 1662,
    RPC_S_INVALID_STRING_BINDING = 1700,
    RPC_S_WRONG_KIND_OF_BINDING = 1701,
    RPC_S_INVALID_BINDING = 1702,
    RPC_S_PROTSEQ_NOT_SUPPORTED = 1703,
    RPC_S_INVALID_RPC_PROTSEQ = 1704,
    RPC_S_INVALID_STRING_UUID = 1705,
    RPC_S_INVALID_ENDPOINT_FORMAT = 1706,
    RPC_S_INVALID_NET_ADDR = 1707,
    RPC_S_NO_ENDPOINT_FOUND = 1708,
    RPC_S_INVALID_TIMEOUT = 1709,
    RPC_S_OBJECT_NOT_FOUND = 1710,
    RPC_S_ALREADY_REGISTERED = 1711,
    RPC_S_TYPE_ALREADY_REGISTERED = 1712,
    RPC_S_ALREADY_LISTENING = 1713,
    RPC_S_NO_PROTSEQS_REGISTERED = 1714,
    RPC_S_NOT_LISTENING = 1715,
    RPC_S_UNKNOWN_MGR_TYPE = 1716,
    RPC_S_UNKNOWN_IF = 1717,
    RPC_S_NO_BINDINGS = 1718,
    RPC_S_NO_PROTSEQS = 1719,
    RPC_S_CANT_CREATE_ENDPOINT = 1720,
    RPC_S_OUT_OF_RESOURCES = 1721,
    RPC_S_SERVER_UNAVAILABLE = 1722,
    RPC_S_SERVER_TOO_BUSY = 1723,
    RPC_S_INVALID_NETWORK_OPTIONS = 1724,
    RPC_S_NO_CALL_ACTIVE = 1725,
    RPC_S_CALL_FAILED = 1726,
    RPC_S_CALL_FAILED_DNE = 1727,
    RPC_S_PROTOCOL_ERROR = 1728,
    RPC_S_PROXY_ACCESS_DENIED = 1729,
    RPC_S_UNSUPPORTED_TRANS_SYN = 1730,
    RPC_S_UNSUPPORTED_TYPE = 1732,
    RPC_S_INVALID_TAG = 1733,
    RPC_S_INVALID_BOUND = 1734,
    RPC_S_NO_ENTRY_NAME = 1735,
    RPC_S_INVALID_NAME_SYNTAX = 1736,
    RPC_S_UNSUPPORTED_NAME_SYNTAX = 1737,
    RPC_S_UUID_NO_ADDRESS = 1739,
    RPC_S_DUPLICATE_ENDPOINT = 1740,
    RPC_S_UNKNOWN_AUTHN_TYPE = 1741,
    RPC_S_MAX_CALLS_TOO_SMALL = 1742,
    RPC_S_STRING_TOO_LONG = 1743,
    RPC_S_PROTSEQ_NOT_FOUND = 1744,
    RPC_S_PROCNUM_OUT_OF_RANGE = 1745,
    RPC_S_BINDING_HAS_NO_AUTH = 1746,
    RPC_S_UNKNOWN_AUTHN_SERVICE = 1747,
    RPC_S_UNKNOWN_AUTHN_LEVEL = 1748,
    RPC_S_INVALID_AUTH_IDENTITY = 1749,
    RPC_S_UNKNOWN_AUTHZ_SERVICE = 1750,
    RPC_S_NOTHING_TO_EXPORT = 1754,
    RPC_S_INCOMPLETE_NAME = 1755,
    RPC_S_INVALID_VERS_OPTION = 1756,
    RPC_S_NO_MORE_MEMBERS = 1757,
    RPC_S_NOT_ALL_OBJS_UNEXPORTED = 1758,
    RPC_S_INTERFACE_NOT_FOUND = 1759,
    RPC_S_ENTRY_ALREADY_EXISTS = 1760,
    RPC_S_ENTRY_NOT_FOUND = 1761,
    RPC_S_NAME_SERVICE_UNAVAILABLE = 1762,
    RPC_S_INVALID_NAF_ID = 1763,
    RPC_S_CANNOT_SUPPORT = 1764,
    RPC_S_NO_CONTEXT_AVAILABLE = 1765,
    RPC_S_INTERNAL_ERROR = 1766,
    RPC_S_ZERO_DIVIDE = 1767,
    RPC_S_ADDRESS_ERROR = 1768,
    RPC_S_FP_DIV_ZERO = 1769,
    RPC_S_FP_UNDERFLOW = 1770,
    RPC_S_FP_OVERFLOW = 1771,
    RPC_X_NO_MORE_ENTRIES = 1772,
    RPC_X_SS_CHAR_TRANS_OPEN_FAIL = 1773,
    RPC_X_SS_CHAR_TRANS_SHORT_FILE = 1774,
    RPC_X_SS_IN_NULL_CONTEXT = 1775,
    RPC_X_SS_CONTEXT_DAMAGED = 1777,
    RPC_X_SS_HANDLES_MISMATCH = 1778,
    RPC_X_SS_CANNOT_GET_CALL_HANDLE = 1779,
    RPC_X_NULL_REF_POINTER = 1780,
    RPC_X_ENUM_VALUE_OUT_OF_RANGE = 1781,
    RPC_X_BYTE_COUNT_TOO_SMALL = 1782,
    RPC_X_BAD_STUB_DATA = 1783,
    ERROR_INVALID_USER_BUFFER = 1784,
    ERROR_UNRECOGNIZED_MEDIA = 1785,
    ERROR_NO_TRUST_LSA_SECRET = 1786,
    ERROR_NO_TRUST_SAM_ACCOUNT = 1787,
    ERROR_TRUSTED_DOMAIN_FAILURE = 1788,
    ERROR_TRUSTED_RELATIONSHIP_FAILURE = 1789,
    ERROR_TRUST_FAILURE = 1790,
    RPC_S_CALL_IN_PROGRESS = 1791,
    ERROR_NETLOGON_NOT_STARTED = 1792,
    ERROR_ACCOUNT_EXPIRED = 1793,
    ERROR_REDIRECTOR_HAS_OPEN_HANDLES = 1794,
    ERROR_PRINTER_DRIVER_ALREADY_INSTALLED = 1795,
    ERROR_UNKNOWN_PORT = 1796,
    ERROR_UNKNOWN_PRINTER_DRIVER = 1797,
    ERROR_UNKNOWN_PRINTPROCESSOR = 1798,
    ERROR_INVALID_SEPARATOR_FILE = 1799,
    ERROR_INVALID_PRIORITY = 1800,
    ERROR_INVALID_PRINTER_NAME = 1801,
    ERROR_PRINTER_ALREADY_EXISTS = 1802,
    ERROR_INVALID_PRINTER_COMMAND = 1803,
    ERROR_INVALID_DATATYPE = 1804,
    ERROR_INVALID_ENVIRONMENT = 1805,
    RPC_S_NO_MORE_BINDINGS = 1806,
    ERROR_NOLOGON_INTERDOMAIN_TRUST_ACCOUNT = 1807,
    ERROR_NOLOGON_WORKSTATION_TRUST_ACCOUNT = 1808,
    ERROR_NOLOGON_SERVER_TRUST_ACCOUNT = 1809,
    ERROR_DOMAIN_TRUST_INCONSISTENT = 1810,
    ERROR_SERVER_HAS_OPEN_HANDLES = 1811,
    ERROR_RESOURCE_DATA_NOT_FOUND = 1812,
    ERROR_RESOURCE_TYPE_NOT_FOUND = 1813,
    ERROR_RESOURCE_NAME_NOT_FOUND = 1814,
    ERROR_RESOURCE_LANG_NOT_FOUND = 1815,
    ERROR_NOT_ENOUGH_QUOTA = 1816,
    RPC_S_NO_INTERFACES = 1817,
    RPC_S_CALL_CANCELLED = 1818,
    RPC_S_BINDING_INCOMPLETE = 1819,
    RPC_S_COMM_FAILURE = 1820,
    RPC_S_UNSUPPORTED_AUTHN_LEVEL = 1821,
    RPC_S_NO_PRINC_NAME = 1822,
    RPC_S_NOT_RPC_ERROR = 1823,
    RPC_S_UUID_LOCAL_ONLY = 1824,
    RPC_S_SEC_PKG_ERROR = 1825,
    RPC_S_NOT_CANCELLED = 1826,
    RPC_X_INVALID_ES_ACTION = 1827,
    RPC_X_WRONG_ES_VERSION = 1828,
    RPC_X_WRONG_STUB_VERSION = 1829,
    RPC_X_INVALID_PIPE_OBJECT = 1830,
    RPC_X_WRONG_PIPE_ORDER = 1831,
    RPC_X_WRONG_PIPE_VERSION = 1832,
    RPC_S_COOKIE_AUTH_FAILED = 1833,
    RPC_S_DO_NOT_DISTURB = 1834,
    RPC_S_SYSTEM_HANDLE_COUNT_EXCEEDED = 1835,
    RPC_S_SYSTEM_HANDLE_TYPE_MISMATCH = 1836,
    RPC_S_GROUP_MEMBER_NOT_FOUND = 1898,
    RPC_S_INVALID_OBJECT = 1900,
    ERROR_INVALID_TIME = 1901,
    ERROR_INVALID_FORM_NAME = 1902,
    ERROR_INVALID_FORM_SIZE = 1903,
    ERROR_ALREADY_WAITING = 1904,
    ERROR_PRINTER_DELETED = 1905,
    ERROR_INVALID_PRINTER_STATE = 1906,
    ERROR_PASSWORD_MUST_CHANGE = 1907,
    ERROR_DOMAIN_CONTROLLER_NOT_FOUND = 1908,
    ERROR_ACCOUNT_LOCKED_OUT = 1909,
    RPC_S_SEND_INCOMPLETE = 1913,
    RPC_S_INVALID_ASYNC_HANDLE = 1914,
    RPC_S_INVALID_ASYNC_CALL = 1915,
    RPC_X_PIPE_CLOSED = 1916,
    RPC_X_PIPE_DISCIPLINE_ERROR = 1917,
    RPC_X_PIPE_EMPTY = 1918,
    ERROR_NO_SITENAME = 1919,
    ERROR_CANT_ACCESS_FILE = 1920,
    ERROR_CANT_RESOLVE_FILENAME = 1921,
    RPC_S_ENTRY_TYPE_MISMATCH = 1922,
    RPC_S_NOT_ALL_OBJS_EXPORTED = 1923,
    RPC_S_INTERFACE_NOT_EXPORTED = 1924,
    RPC_S_PROFILE_NOT_ADDED = 1925,
    RPC_S_PRF_ELT_NOT_ADDED = 1926,
    RPC_S_PRF_ELT_NOT_REMOVED = 1927,
    RPC_S_GRP_ELT_NOT_ADDED = 1928,
    RPC_S_GRP_ELT_NOT_REMOVED = 1929,
    ERROR_KM_DRIVER_BLOCKED = 1930,
    ERROR_CONTEXT_EXPIRED = 1931,
    ERROR_PER_USER_TRUST_QUOTA_EXCEEDED = 1932,
    ERROR_ALL_USER_TRUST_QUOTA_EXCEEDED = 1933,
    ERROR_USER_DELETE_TRUST_QUOTA_EXCEEDED = 1934,
    ERROR_AUTHENTICATION_FIREWALL_FAILED = 1935,
    ERROR_REMOTE_PRINT_CONNECTIONS_BLOCKED = 1936,
    ERROR_NTLM_BLOCKED = 1937,
    ERROR_PASSWORD_CHANGE_REQUIRED = 1938,
    ERROR_LOST_MODE_LOGON_RESTRICTION = 1939,
    ERROR_INVALID_PIXEL_FORMAT = 2000,
    ERROR_BAD_DRIVER = 2001,
    ERROR_INVALID_WINDOW_STYLE = 2002,
    ERROR_METAFILE_NOT_SUPPORTED = 2003,
    ERROR_TRANSFORM_NOT_SUPPORTED = 2004,
    ERROR_CLIPPING_NOT_SUPPORTED = 2005,
    ERROR_INVALID_CMM = 2010,
    ERROR_INVALID_PROFILE = 2011,
    ERROR_TAG_NOT_FOUND = 2012,
    ERROR_TAG_NOT_PRESENT = 2013,
    ERROR_DUPLICATE_TAG = 2014,
    ERROR_PROFILE_NOT_ASSOCIATED_WITH_DEVICE = 2015,
    ERROR_PROFILE_NOT_FOUND = 2016,
    ERROR_INVALID_COLORSPACE = 2017,
    ERROR_ICM_NOT_ENABLED = 2018,
    ERROR_DELETING_ICM_XFORM = 2019,
    ERROR_INVALID_TRANSFORM = 2020,
    ERROR_COLORSPACE_MISMATCH = 2021,
    ERROR_INVALID_COLORINDEX = 2022,
    ERROR_PROFILE_DOES_NOT_MATCH_DEVICE = 2023,
    ERROR_CONNECTED_OTHER_PASSWORD = 2108,
    ERROR_CONNECTED_OTHER_PASSWORD_DEFAULT = 2109,
    ERROR_BAD_USERNAME = 2202,
    ERROR_NOT_CONNECTED = 2250,
    ERROR_OPEN_FILES = 2401,
    ERROR_ACTIVE_CONNECTIONS = 2402,
    ERROR_DEVICE_IN_USE = 2404,
    ERROR_UNKNOWN_PRINT_MONITOR = 3000,
    ERROR_PRINTER_DRIVER_IN_USE = 3001,
    ERROR_SPOOL_FILE_NOT_FOUND = 3002,
    ERROR_SPL_NO_STARTDOC = 3003,
    ERROR_SPL_NO_ADDJOB = 3004,
    ERROR_PRINT_PROCESSOR_ALREADY_INSTALLED = 3005,
    ERROR_PRINT_MONITOR_ALREADY_INSTALLED = 3006,
    ERROR_INVALID_PRINT_MONITOR = 3007,
    ERROR_PRINT_MONITOR_IN_USE = 3008,
    ERROR_PRINTER_HAS_JOBS_QUEUED = 3009,
    ERROR_SUCCESS_REBOOT_REQUIRED = 3010,
    ERROR_SUCCESS_RESTART_REQUIRED = 3011,
    ERROR_PRINTER_NOT_FOUND = 3012,
    ERROR_PRINTER_DRIVER_WARNED = 3013,
    ERROR_PRINTER_DRIVER_BLOCKED = 3014,
    ERROR_PRINTER_DRIVER_PACKAGE_IN_USE = 3015,
    ERROR_CORE_DRIVER_PACKAGE_NOT_FOUND = 3016,
    ERROR_FAIL_REBOOT_REQUIRED = 3017,
    ERROR_FAIL_REBOOT_INITIATED = 3018,
    ERROR_PRINTER_DRIVER_DOWNLOAD_NEEDED = 3019,
    ERROR_PRINT_JOB_RESTART_REQUIRED = 3020,
    ERROR_INVALID_PRINTER_DRIVER_MANIFEST = 3021,
    ERROR_PRINTER_NOT_SHAREABLE = 3022,
    ERROR_REQUEST_PAUSED = 3050,
    ERROR_APPEXEC_CONDITION_NOT_SATISFIED = 3060,
    ERROR_APPEXEC_HANDLE_INVALIDATED = 3061,
    ERROR_APPEXEC_INVALID_HOST_GENERATION = 3062,
    ERROR_APPEXEC_UNEXPECTED_PROCESS_REGISTRATION = 3063,
    ERROR_APPEXEC_INVALID_HOST_STATE = 3064,
    ERROR_APPEXEC_NO_DONOR = 3065,
    ERROR_APPEXEC_HOST_ID_MISMATCH = 3066,
    ERROR_APPEXEC_UNKNOWN_USER = 3067,
    ERROR_IO_REISSUE_AS_CACHED = 3950,
    ERROR_WINS_INTERNAL = 4000,
    ERROR_CAN_NOT_DEL_LOCAL_WINS = 4001,
    ERROR_STATIC_INIT = 4002,
    ERROR_INC_BACKUP = 4003,
    ERROR_FULL_BACKUP = 4004,
    ERROR_REC_NON_EXISTENT = 4005,
    ERROR_RPL_NOT_ALLOWED = 4006,
    ERROR_DHCP_ADDRESS_CONFLICT = 4100,
    ERROR_WMI_GUID_NOT_FOUND = 4200,
    ERROR_WMI_INSTANCE_NOT_FOUND = 4201,
    ERROR_WMI_ITEMID_NOT_FOUND = 4202,
    ERROR_WMI_TRY_AGAIN = 4203,
    ERROR_WMI_DP_NOT_FOUND = 4204,
    ERROR_WMI_UNRESOLVED_INSTANCE_REF = 4205,
    ERROR_WMI_ALREADY_ENABLED = 4206,
    ERROR_WMI_GUID_DISCONNECTED = 4207,
    ERROR_WMI_SERVER_UNAVAILABLE = 4208,
    ERROR_WMI_DP_FAILED = 4209,
    ERROR_WMI_INVALID_MOF = 4210,
    ERROR_WMI_INVALID_REGINFO = 4211,
    ERROR_WMI_ALREADY_DISABLED = 4212,
    ERROR_WMI_READ_ONLY = 4213,
    ERROR_WMI_SET_FAILURE = 4214,
    ERROR_NOT_APPCONTAINER = 4250,
    ERROR_APPCONTAINER_REQUIRED = 4251,
    ERROR_NOT_SUPPORTED_IN_APPCONTAINER = 4252,
    ERROR_INVALID_PACKAGE_SID_LENGTH = 4253,
    ERROR_INVALID_MEDIA = 4300,
    ERROR_INVALID_LIBRARY = 4301,
    ERROR_INVALID_MEDIA_POOL = 4302,
    ERROR_DRIVE_MEDIA_MISMATCH = 4303,
    ERROR_MEDIA_OFFLINE = 4304,
    ERROR_LIBRARY_OFFLINE = 4305,
    ERROR_EMPTY = 4306,
    ERROR_NOT_EMPTY = 4307,
    ERROR_MEDIA_UNAVAILABLE = 4308,
    ERROR_RESOURCE_DISABLED = 4309,
    ERROR_INVALID_CLEANER = 4310,
    ERROR_UNABLE_TO_CLEAN = 4311,
    ERROR_OBJECT_NOT_FOUND = 4312,
    ERROR_DATABASE_FAILURE = 4313,
    ERROR_DATABASE_FULL = 4314,
    ERROR_MEDIA_INCOMPATIBLE = 4315,
    ERROR_RESOURCE_NOT_PRESENT = 4316,
    ERROR_INVALID_OPERATION = 4317,
    ERROR_MEDIA_NOT_AVAILABLE = 4318,
    ERROR_DEVICE_NOT_AVAILABLE = 4319,
    ERROR_REQUEST_REFUSED = 4320,
    ERROR_INVALID_DRIVE_OBJECT = 4321,
    ERROR_LIBRARY_FULL = 4322,
    ERROR_MEDIUM_NOT_ACCESSIBLE = 4323,
    ERROR_UNABLE_TO_LOAD_MEDIUM = 4324,
    ERROR_UNABLE_TO_INVENTORY_DRIVE = 4325,
    ERROR_UNABLE_TO_INVENTORY_SLOT = 4326,
    ERROR_UNABLE_TO_INVENTORY_TRANSPORT = 4327,
    ERROR_TRANSPORT_FULL = 4328,
    ERROR_CONTROLLING_IEPORT = 4329,
    ERROR_UNABLE_TO_EJECT_MOUNTED_MEDIA = 4330,
    ERROR_CLEANER_SLOT_SET = 4331,
    ERROR_CLEANER_SLOT_NOT_SET = 4332,
    ERROR_CLEANER_CARTRIDGE_SPENT = 4333,
    ERROR_UNEXPECTED_OMID = 4334,
    ERROR_CANT_DELETE_LAST_ITEM = 4335,
    ERROR_MESSAGE_EXCEEDS_MAX_SIZE = 4336,
    ERROR_VOLUME_CONTAINS_SYS_FILES = 4337,
    ERROR_INDIGENOUS_TYPE = 4338,
    ERROR_NO_SUPPORTING_DRIVES = 4339,
    ERROR_CLEANER_CARTRIDGE_INSTALLED = 4340,
    ERROR_IEPORT_FULL = 4341,
    ERROR_FILE_OFFLINE = 4350,
    ERROR_REMOTE_STORAGE_NOT_ACTIVE = 4351,
    ERROR_REMOTE_STORAGE_MEDIA_ERROR = 4352,
    ERROR_NOT_A_REPARSE_POINT = 4390,
    ERROR_REPARSE_ATTRIBUTE_CONFLICT = 4391,
    ERROR_INVALID_REPARSE_DATA = 4392,
    ERROR_REPARSE_TAG_INVALID = 4393,
    ERROR_REPARSE_TAG_MISMATCH = 4394,
    ERROR_REPARSE_POINT_ENCOUNTERED = 4395,
    ERROR_APP_DATA_NOT_FOUND = 4400,
    ERROR_APP_DATA_EXPIRED = 4401,
    ERROR_APP_DATA_CORRUPT = 4402,
    ERROR_APP_DATA_LIMIT_EXCEEDED = 4403,
    ERROR_APP_DATA_REBOOT_REQUIRED = 4404,
    ERROR_SECUREBOOT_ROLLBACK_DETECTED = 4420,
    ERROR_SECUREBOOT_POLICY_VIOLATION = 4421,
    ERROR_SECUREBOOT_INVALID_POLICY = 4422,
    ERROR_SECUREBOOT_POLICY_PUBLISHER_NOT_FOUND = 4423,
    ERROR_SECUREBOOT_POLICY_NOT_SIGNED = 4424,
    ERROR_SECUREBOOT_NOT_ENABLED = 4425,
    ERROR_SECUREBOOT_FILE_REPLACED = 4426,
    ERROR_SECUREBOOT_POLICY_NOT_AUTHORIZED = 4427,
    ERROR_SECUREBOOT_POLICY_UNKNOWN = 4428,
    ERROR_SECUREBOOT_POLICY_MISSING_ANTIROLLBACKVERSION = 4429,
    ERROR_SECUREBOOT_PLATFORM_ID_MISMATCH = 4430,
    ERROR_SECUREBOOT_POLICY_ROLLBACK_DETECTED = 4431,
    ERROR_SECUREBOOT_POLICY_UPGRADE_MISMATCH = 4432,
    ERROR_SECUREBOOT_REQUIRED_POLICY_FILE_MISSING = 4433,
    ERROR_SECUREBOOT_NOT_BASE_POLICY = 4434,
    ERROR_SECUREBOOT_NOT_SUPPLEMENTAL_POLICY = 4435,
    ERROR_OFFLOAD_READ_FLT_NOT_SUPPORTED = 4440,
    ERROR_OFFLOAD_WRITE_FLT_NOT_SUPPORTED = 4441,
    ERROR_OFFLOAD_READ_FILE_NOT_SUPPORTED = 4442,
    ERROR_OFFLOAD_WRITE_FILE_NOT_SUPPORTED = 4443,
    ERROR_ALREADY_HAS_STREAM_ID = 4444,
    ERROR_SMR_GARBAGE_COLLECTION_REQUIRED = 4445,
    ERROR_WOF_WIM_HEADER_CORRUPT = 4446,
    ERROR_WOF_WIM_RESOURCE_TABLE_CORRUPT = 4447,
    ERROR_WOF_FILE_RESOURCE_TABLE_CORRUPT = 4448,
    ERROR_VOLUME_NOT_SIS_ENABLED = 4500,
    ERROR_SYSTEM_INTEGRITY_ROLLBACK_DETECTED = 4550,
    ERROR_SYSTEM_INTEGRITY_POLICY_VIOLATION = 4551,
    ERROR_SYSTEM_INTEGRITY_INVALID_POLICY = 4552,
    ERROR_SYSTEM_INTEGRITY_POLICY_NOT_SIGNED = 4553,
    ERROR_SYSTEM_INTEGRITY_TOO_MANY_POLICIES = 4554,
    ERROR_SYSTEM_INTEGRITY_SUPPLEMENTAL_POLICY_NOT_AUTHORIZED = 4555,
    ERROR_VSM_NOT_INITIALIZED = 4560,
    ERROR_VSM_DMA_PROTECTION_NOT_IN_USE = 4561,
    ERROR_PLATFORM_MANIFEST_NOT_AUTHORIZED = 4570,
    ERROR_PLATFORM_MANIFEST_INVALID = 4571,
    ERROR_PLATFORM_MANIFEST_FILE_NOT_AUTHORIZED = 4572,
    ERROR_PLATFORM_MANIFEST_CATALOG_NOT_AUTHORIZED = 4573,
    ERROR_PLATFORM_MANIFEST_BINARY_ID_NOT_FOUND = 4574,
    ERROR_PLATFORM_MANIFEST_NOT_ACTIVE = 4575,
    ERROR_PLATFORM_MANIFEST_NOT_SIGNED = 4576,
    ERROR_DEPENDENT_RESOURCE_EXISTS = 5001,
    ERROR_DEPENDENCY_NOT_FOUND = 5002,
    ERROR_DEPENDENCY_ALREADY_EXISTS = 5003,
    ERROR_RESOURCE_NOT_ONLINE = 5004,
    ERROR_HOST_NODE_NOT_AVAILABLE = 5005,
    ERROR_RESOURCE_NOT_AVAILABLE = 5006,
    ERROR_RESOURCE_NOT_FOUND = 5007,
    ERROR_SHUTDOWN_CLUSTER = 5008,
    ERROR_CANT_EVICT_ACTIVE_NODE = 5009,
    ERROR_OBJECT_ALREADY_EXISTS = 5010,
    ERROR_OBJECT_IN_LIST = 5011,
    ERROR_GROUP_NOT_AVAILABLE = 5012,
    ERROR_GROUP_NOT_FOUND = 5013,
    ERROR_GROUP_NOT_ONLINE = 5014,
    ERROR_HOST_NODE_NOT_RESOURCE_OWNER = 5015,
    ERROR_HOST_NODE_NOT_GROUP_OWNER = 5016,
    ERROR_RESMON_CREATE_FAILED = 5017,
    ERROR_RESMON_ONLINE_FAILED = 5018,
    ERROR_RESOURCE_ONLINE = 5019,
    ERROR_QUORUM_RESOURCE = 5020,
    ERROR_NOT_QUORUM_CAPABLE = 5021,
    ERROR_CLUSTER_SHUTTING_DOWN = 5022,
    ERROR_INVALID_STATE = 5023,
    ERROR_RESOURCE_PROPERTIES_STORED = 5024,
    ERROR_NOT_QUORUM_CLASS = 5025,
    ERROR_CORE_RESOURCE = 5026,
    ERROR_QUORUM_RESOURCE_ONLINE_FAILED = 5027,
    ERROR_QUORUMLOG_OPEN_FAILED = 5028,
    ERROR_CLUSTERLOG_CORRUPT = 5029,
    ERROR_CLUSTERLOG_RECORD_EXCEEDS_MAXSIZE = 5030,
    ERROR_CLUSTERLOG_EXCEEDS_MAXSIZE = 5031,
    ERROR_CLUSTERLOG_CHKPOINT_NOT_FOUND = 5032,
    ERROR_CLUSTERLOG_NOT_ENOUGH_SPACE = 5033,
    ERROR_QUORUM_OWNER_ALIVE = 5034,
    ERROR_NETWORK_NOT_AVAILABLE = 5035,
    ERROR_NODE_NOT_AVAILABLE = 5036,
    ERROR_ALL_NODES_NOT_AVAILABLE = 5037,
    ERROR_RESOURCE_FAILED = 5038,
    ERROR_CLUSTER_INVALID_NODE = 5039,
    ERROR_CLUSTER_NODE_EXISTS = 5040,
    ERROR_CLUSTER_JOIN_IN_PROGRESS = 5041,
    ERROR_CLUSTER_NODE_NOT_FOUND = 5042,
    ERROR_CLUSTER_LOCAL_NODE_NOT_FOUND = 5043,
    ERROR_CLUSTER_NETWORK_EXISTS = 5044,
    ERROR_CLUSTER_NETWORK_NOT_FOUND = 5045,
    ERROR_CLUSTER_NETINTERFACE_EXISTS = 5046,
    ERROR_CLUSTER_NETINTERFACE_NOT_FOUND = 5047,
    ERROR_CLUSTER_INVALID_REQUEST = 5048,
    ERROR_CLUSTER_INVALID_NETWORK_PROVIDER = 5049,
    ERROR_CLUSTER_NODE_DOWN = 5050,
    ERROR_CLUSTER_NODE_UNREACHABLE = 5051,
    ERROR_CLUSTER_NODE_NOT_MEMBER = 5052,
    ERROR_CLUSTER_JOIN_NOT_IN_PROGRESS = 5053,
    ERROR_CLUSTER_INVALID_NETWORK = 5054,
    ERROR_CLUSTER_NODE_UP = 5056,
    ERROR_CLUSTER_IPADDR_IN_USE = 5057,
    ERROR_CLUSTER_NODE_NOT_PAUSED = 5058,
    ERROR_CLUSTER_NO_SECURITY_CONTEXT = 5059,
    ERROR_CLUSTER_NETWORK_NOT_INTERNAL = 5060,
    ERROR_CLUSTER_NODE_ALREADY_UP = 5061,
    ERROR_CLUSTER_NODE_ALREADY_DOWN = 5062,
    ERROR_CLUSTER_NETWORK_ALREADY_ONLINE = 5063,
    ERROR_CLUSTER_NETWORK_ALREADY_OFFLINE = 5064,
    ERROR_CLUSTER_NODE_ALREADY_MEMBER = 5065,
    ERROR_CLUSTER_LAST_INTERNAL_NETWORK = 5066,
    ERROR_CLUSTER_NETWORK_HAS_DEPENDENTS = 5067,
    ERROR_INVALID_OPERATION_ON_QUORUM = 5068,
    ERROR_DEPENDENCY_NOT_ALLOWED = 5069,
    ERROR_CLUSTER_NODE_PAUSED = 5070,
    ERROR_NODE_CANT_HOST_RESOURCE = 5071,
    ERROR_CLUSTER_NODE_NOT_READY = 5072,
    ERROR_CLUSTER_NODE_SHUTTING_DOWN = 5073,
    ERROR_CLUSTER_JOIN_ABORTED = 5074,
    ERROR_CLUSTER_INCOMPATIBLE_VERSIONS = 5075,
    ERROR_CLUSTER_MAXNUM_OF_RESOURCES_EXCEEDED = 5076,
    ERROR_CLUSTER_SYSTEM_CONFIG_CHANGED = 5077,
    ERROR_CLUSTER_RESOURCE_TYPE_NOT_FOUND = 5078,
    ERROR_CLUSTER_RESTYPE_NOT_SUPPORTED = 5079,
    ERROR_CLUSTER_RESNAME_NOT_FOUND = 5080,
    ERROR_CLUSTER_NO_RPC_PACKAGES_REGISTERED = 5081,
    ERROR_CLUSTER_OWNER_NOT_IN_PREFLIST = 5082,
    ERROR_CLUSTER_DATABASE_SEQMISMATCH = 5083,
    ERROR_RESMON_INVALID_STATE = 5084,
    ERROR_CLUSTER_GUM_NOT_LOCKER = 5085,
    ERROR_QUORUM_DISK_NOT_FOUND = 5086,
    ERROR_DATABASE_BACKUP_CORRUPT = 5087,
    ERROR_CLUSTER_NODE_ALREADY_HAS_DFS_ROOT = 5088,
    ERROR_RESOURCE_PROPERTY_UNCHANGEABLE = 5089,
    ERROR_NO_ADMIN_ACCESS_POINT = 5090,
    ERROR_CLUSTER_MEMBERSHIP_INVALID_STATE = 5890,
    ERROR_CLUSTER_QUORUMLOG_NOT_FOUND = 5891,
    ERROR_CLUSTER_MEMBERSHIP_HALT = 5892,
    ERROR_CLUSTER_INSTANCE_ID_MISMATCH = 5893,
    ERROR_CLUSTER_NETWORK_NOT_FOUND_FOR_IP = 5894,
    ERROR_CLUSTER_PROPERTY_DATA_TYPE_MISMATCH = 5895,
    ERROR_CLUSTER_EVICT_WITHOUT_CLEANUP = 5896,
    ERROR_CLUSTER_PARAMETER_MISMATCH = 5897,
    ERROR_NODE_CANNOT_BE_CLUSTERED = 5898,
    ERROR_CLUSTER_WRONG_OS_VERSION = 5899,
    ERROR_CLUSTER_CANT_CREATE_DUP_CLUSTER_NAME = 5900,
    ERROR_CLUSCFG_ALREADY_COMMITTED = 5901,
    ERROR_CLUSCFG_ROLLBACK_FAILED = 5902,
    ERROR_CLUSCFG_SYSTEM_DISK_DRIVE_LETTER_CONFLICT = 5903,
    ERROR_CLUSTER_OLD_VERSION = 5904,
    ERROR_CLUSTER_MISMATCHED_COMPUTER_ACCT_NAME = 5905,
    ERROR_CLUSTER_NO_NET_ADAPTERS = 5906,
    ERROR_CLUSTER_POISONED = 5907,
    ERROR_CLUSTER_GROUP_MOVING = 5908,
    ERROR_CLUSTER_RESOURCE_TYPE_BUSY = 5909,
    ERROR_RESOURCE_CALL_TIMED_OUT = 5910,
    ERROR_INVALID_CLUSTER_IPV6_ADDRESS = 5911,
    ERROR_CLUSTER_INTERNAL_INVALID_FUNCTION = 5912,
    ERROR_CLUSTER_PARAMETER_OUT_OF_BOUNDS = 5913,
    ERROR_CLUSTER_PARTIAL_SEND = 5914,
    ERROR_CLUSTER_REGISTRY_INVALID_FUNCTION = 5915,
    ERROR_CLUSTER_INVALID_STRING_TERMINATION = 5916,
    ERROR_CLUSTER_INVALID_STRING_FORMAT = 5917,
    ERROR_CLUSTER_DATABASE_TRANSACTION_IN_PROGRESS = 5918,
    ERROR_CLUSTER_DATABASE_TRANSACTION_NOT_IN_PROGRESS = 5919,
    ERROR_CLUSTER_NULL_DATA = 5920,
    ERROR_CLUSTER_PARTIAL_READ = 5921,
    ERROR_CLUSTER_PARTIAL_WRITE = 5922,
    ERROR_CLUSTER_CANT_DESERIALIZE_DATA = 5923,
    ERROR_DEPENDENT_RESOURCE_PROPERTY_CONFLICT = 5924,
    ERROR_CLUSTER_NO_QUORUM = 5925,
    ERROR_CLUSTER_INVALID_IPV6_NETWORK = 5926,
    ERROR_CLUSTER_INVALID_IPV6_TUNNEL_NETWORK = 5927,
    ERROR_QUORUM_NOT_ALLOWED_IN_THIS_GROUP = 5928,
    ERROR_DEPENDENCY_TREE_TOO_COMPLEX = 5929,
    ERROR_EXCEPTION_IN_RESOURCE_CALL = 5930,
    ERROR_CLUSTER_RHS_FAILED_INITIALIZATION = 5931,
    ERROR_CLUSTER_NOT_INSTALLED = 5932,
    ERROR_CLUSTER_RESOURCES_MUST_BE_ONLINE_ON_THE_SAME_NODE = 5933,
    ERROR_CLUSTER_MAX_NODES_IN_CLUSTER = 5934,
    ERROR_CLUSTER_TOO_MANY_NODES = 5935,
    ERROR_CLUSTER_OBJECT_ALREADY_USED = 5936,
    ERROR_NONCORE_GROUPS_FOUND = 5937,
    ERROR_FILE_SHARE_RESOURCE_CONFLICT = 5938,
    ERROR_CLUSTER_EVICT_INVALID_REQUEST = 5939,
    ERROR_CLUSTER_SINGLETON_RESOURCE = 5940,
    ERROR_CLUSTER_GROUP_SINGLETON_RESOURCE = 5941,
    ERROR_CLUSTER_RESOURCE_PROVIDER_FAILED = 5942,
    ERROR_CLUSTER_RESOURCE_CONFIGURATION_ERROR = 5943,
    ERROR_CLUSTER_GROUP_BUSY = 5944,
    ERROR_CLUSTER_NOT_SHARED_VOLUME = 5945,
    ERROR_CLUSTER_INVALID_SECURITY_DESCRIPTOR = 5946,
    ERROR_CLUSTER_SHARED_VOLUMES_IN_USE = 5947,
    ERROR_CLUSTER_USE_SHARED_VOLUMES_API = 5948,
    ERROR_CLUSTER_BACKUP_IN_PROGRESS = 5949,
    ERROR_NON_CSV_PATH = 5950,
    ERROR_CSV_VOLUME_NOT_LOCAL = 5951,
    ERROR_CLUSTER_WATCHDOG_TERMINATING = 5952,
    ERROR_CLUSTER_RESOURCE_VETOED_MOVE_INCOMPATIBLE_NODES = 5953,
    ERROR_CLUSTER_INVALID_NODE_WEIGHT = 5954,
    ERROR_CLUSTER_RESOURCE_VETOED_CALL = 5955,
    ERROR_RESMON_SYSTEM_RESOURCES_LACKING = 5956,
    ERROR_CLUSTER_RESOURCE_VETOED_MOVE_NOT_ENOUGH_RESOURCES_ON_DESTINATION = 5957,
    ERROR_CLUSTER_RESOURCE_VETOED_MOVE_NOT_ENOUGH_RESOURCES_ON_SOURCE = 5958,
    ERROR_CLUSTER_GROUP_QUEUED = 5959,
    ERROR_CLUSTER_RESOURCE_LOCKED_STATUS = 5960,
    ERROR_CLUSTER_SHARED_VOLUME_FAILOVER_NOT_ALLOWED = 5961,
    ERROR_CLUSTER_NODE_DRAIN_IN_PROGRESS = 5962,
    ERROR_CLUSTER_DISK_NOT_CONNECTED = 5963,
    ERROR_DISK_NOT_CSV_CAPABLE = 5964,
    ERROR_RESOURCE_NOT_IN_AVAILABLE_STORAGE = 5965,
    ERROR_CLUSTER_SHARED_VOLUME_REDIRECTED = 5966,
    ERROR_CLUSTER_SHARED_VOLUME_NOT_REDIRECTED = 5967,
    ERROR_CLUSTER_CANNOT_RETURN_PROPERTIES = 5968,
    ERROR_CLUSTER_RESOURCE_CONTAINS_UNSUPPORTED_DIFF_AREA_FOR_SHARED_VOLUMES = 5969,
    ERROR_CLUSTER_RESOURCE_IS_IN_MAINTENANCE_MODE = 5970,
    ERROR_CLUSTER_AFFINITY_CONFLICT = 5971,
    ERROR_CLUSTER_RESOURCE_IS_REPLICA_VIRTUAL_MACHINE = 5972,
    ERROR_CLUSTER_UPGRADE_INCOMPATIBLE_VERSIONS = 5973,
    ERROR_CLUSTER_UPGRADE_FIX_QUORUM_NOT_SUPPORTED = 5974,
    ERROR_CLUSTER_UPGRADE_RESTART_REQUIRED = 5975,
    ERROR_CLUSTER_UPGRADE_IN_PROGRESS = 5976,
    ERROR_CLUSTER_UPGRADE_INCOMPLETE = 5977,
    ERROR_CLUSTER_NODE_IN_GRACE_PERIOD = 5978,
    ERROR_CLUSTER_CSV_IO_PAUSE_TIMEOUT = 5979,
    ERROR_NODE_NOT_ACTIVE_CLUSTER_MEMBER = 5980,
    ERROR_CLUSTER_RESOURCE_NOT_MONITORED = 5981,
    ERROR_CLUSTER_RESOURCE_DOES_NOT_SUPPORT_UNMONITORED = 5982,
    ERROR_CLUSTER_RESOURCE_IS_REPLICATED = 5983,
    ERROR_CLUSTER_NODE_ISOLATED = 5984,
    ERROR_CLUSTER_NODE_QUARANTINED = 5985,
    ERROR_CLUSTER_DATABASE_UPDATE_CONDITION_FAILED = 5986,
    ERROR_CLUSTER_SPACE_DEGRADED = 5987,
    ERROR_CLUSTER_TOKEN_DELEGATION_NOT_SUPPORTED = 5988,
    ERROR_CLUSTER_CSV_INVALID_HANDLE = 5989,
    ERROR_CLUSTER_CSV_SUPPORTED_ONLY_ON_COORDINATOR = 5990,
    ERROR_GROUPSET_NOT_AVAILABLE = 5991,
    ERROR_GROUPSET_NOT_FOUND = 5992,
    ERROR_GROUPSET_CANT_PROVIDE = 5993,
    ERROR_CLUSTER_FAULT_DOMAIN_PARENT_NOT_FOUND = 5994,
    ERROR_CLUSTER_FAULT_DOMAIN_INVALID_HIERARCHY = 5995,
    ERROR_CLUSTER_FAULT_DOMAIN_FAILED_S2D_VALIDATION = 5996,
    ERROR_CLUSTER_FAULT_DOMAIN_S2D_CONNECTIVITY_LOSS = 5997,
    ERROR_CLUSTER_INVALID_INFRASTRUCTURE_FILESERVER_NAME = 5998,
    ERROR_CLUSTERSET_MANAGEMENT_CLUSTER_UNREACHABLE = 5999,
    ERROR_ENCRYPTION_FAILED = 6000,
    ERROR_DECRYPTION_FAILED = 6001,
    ERROR_FILE_ENCRYPTED = 6002,
    ERROR_NO_RECOVERY_POLICY = 6003,
    ERROR_NO_EFS = 6004,
    ERROR_WRONG_EFS = 6005,
    ERROR_NO_USER_KEYS = 6006,
    ERROR_FILE_NOT_ENCRYPTED = 6007,
    ERROR_NOT_EXPORT_FORMAT = 6008,
    ERROR_FILE_READ_ONLY = 6009,
    ERROR_DIR_EFS_DISALLOWED = 6010,
    ERROR_EFS_SERVER_NOT_TRUSTED = 6011,
    ERROR_BAD_RECOVERY_POLICY = 6012,
    ERROR_EFS_ALG_BLOB_TOO_BIG = 6013,
    ERROR_VOLUME_NOT_SUPPORT_EFS = 6014,
    ERROR_EFS_DISABLED = 6015,
    ERROR_EFS_VERSION_NOT_SUPPORT = 6016,
    ERROR_CS_ENCRYPTION_INVALID_SERVER_RESPONSE = 6017,
    ERROR_CS_ENCRYPTION_UNSUPPORTED_SERVER = 6018,
    ERROR_CS_ENCRYPTION_EXISTING_ENCRYPTED_FILE = 6019,
    ERROR_CS_ENCRYPTION_NEW_ENCRYPTED_FILE = 6020,
    ERROR_CS_ENCRYPTION_FILE_NOT_CSE = 6021,
    ERROR_ENCRYPTION_POLICY_DENIES_OPERATION = 6022,
    ERROR_WIP_ENCRYPTION_FAILED = 6023,
    ERROR_NO_BROWSER_SERVERS_FOUND = 6118,
    ERROR_CLUSTER_OBJECT_IS_CLUSTER_SET_VM = 6250,
    ERROR_LOG_SECTOR_INVALID = 6600,
    ERROR_LOG_SECTOR_PARITY_INVALID = 6601,
    ERROR_LOG_SECTOR_REMAPPED = 6602,
    ERROR_LOG_BLOCK_INCOMPLETE = 6603,
    ERROR_LOG_INVALID_RANGE = 6604,
    ERROR_LOG_BLOCKS_EXHAUSTED = 6605,
    ERROR_LOG_READ_CONTEXT_INVALID = 6606,
    ERROR_LOG_RESTART_INVALID = 6607,
    ERROR_LOG_BLOCK_VERSION = 6608,
    ERROR_LOG_BLOCK_INVALID = 6609,
    ERROR_LOG_READ_MODE_INVALID = 6610,
    ERROR_LOG_NO_RESTART = 6611,
    ERROR_LOG_METADATA_CORRUPT = 6612,
    ERROR_LOG_METADATA_INVALID = 6613,
    ERROR_LOG_METADATA_INCONSISTENT = 6614,
    ERROR_LOG_RESERVATION_INVALID = 6615,
    ERROR_LOG_CANT_DELETE = 6616,
    ERROR_LOG_CONTAINER_LIMIT_EXCEEDED = 6617,
    ERROR_LOG_START_OF_LOG = 6618,
    ERROR_LOG_POLICY_ALREADY_INSTALLED = 6619,
    ERROR_LOG_POLICY_NOT_INSTALLED = 6620,
    ERROR_LOG_POLICY_INVALID = 6621,
    ERROR_LOG_POLICY_CONFLICT = 6622,
    ERROR_LOG_PINNED_ARCHIVE_TAIL = 6623,
    ERROR_LOG_RECORD_NONEXISTENT = 6624,
    ERROR_LOG_RECORDS_RESERVED_INVALID = 6625,
    ERROR_LOG_SPACE_RESERVED_INVALID = 6626,
    ERROR_LOG_TAIL_INVALID = 6627,
    ERROR_LOG_FULL = 6628,
    ERROR_COULD_NOT_RESIZE_LOG = 6629,
    ERROR_LOG_MULTIPLEXED = 6630,
    ERROR_LOG_DEDICATED = 6631,
    ERROR_LOG_ARCHIVE_NOT_IN_PROGRESS = 6632,
    ERROR_LOG_ARCHIVE_IN_PROGRESS = 6633,
    ERROR_LOG_EPHEMERAL = 6634,
    ERROR_LOG_NOT_ENOUGH_CONTAINERS = 6635,
    ERROR_LOG_CLIENT_ALREADY_REGISTERED = 6636,
    ERROR_LOG_CLIENT_NOT_REGISTERED = 6637,
    ERROR_LOG_FULL_HANDLER_IN_PROGRESS = 6638,
    ERROR_LOG_CONTAINER_READ_FAILED = 6639,
    ERROR_LOG_CONTAINER_WRITE_FAILED = 6640,
    ERROR_LOG_CONTAINER_OPEN_FAILED = 6641,
    ERROR_LOG_CONTAINER_STATE_INVALID = 6642,
    ERROR_LOG_STATE_INVALID = 6643,
    ERROR_LOG_PINNED = 6644,
    ERROR_LOG_METADATA_FLUSH_FAILED = 6645,
    ERROR_LOG_INCONSISTENT_SECURITY = 6646,
    ERROR_LOG_APPENDED_FLUSH_FAILED = 6647,
    ERROR_LOG_PINNED_RESERVATION = 6648,
    ERROR_INVALID_TRANSACTION = 6700,
    ERROR_TRANSACTION_NOT_ACTIVE = 6701,
    ERROR_TRANSACTION_REQUEST_NOT_VALID = 6702,
    ERROR_TRANSACTION_NOT_REQUESTED = 6703,
    ERROR_TRANSACTION_ALREADY_ABORTED = 6704,
    ERROR_TRANSACTION_ALREADY_COMMITTED = 6705,
    ERROR_TM_INITIALIZATION_FAILED = 6706,
    ERROR_RESOURCEMANAGER_READ_ONLY = 6707,
    ERROR_TRANSACTION_NOT_JOINED = 6708,
    ERROR_TRANSACTION_SUPERIOR_EXISTS = 6709,
    ERROR_CRM_PROTOCOL_ALREADY_EXISTS = 6710,
    ERROR_TRANSACTION_PROPAGATION_FAILED = 6711,
    ERROR_CRM_PROTOCOL_NOT_FOUND = 6712,
    ERROR_TRANSACTION_INVALID_MARSHALL_BUFFER = 6713,
    ERROR_CURRENT_TRANSACTION_NOT_VALID = 6714,
    ERROR_TRANSACTION_NOT_FOUND = 6715,
    ERROR_RESOURCEMANAGER_NOT_FOUND = 6716,
    ERROR_ENLISTMENT_NOT_FOUND = 6717,
    ERROR_TRANSACTIONMANAGER_NOT_FOUND = 6718,
    ERROR_TRANSACTIONMANAGER_NOT_ONLINE = 6719,
    ERROR_TRANSACTIONMANAGER_RECOVERY_NAME_COLLISION = 6720,
    ERROR_TRANSACTION_NOT_ROOT = 6721,
    ERROR_TRANSACTION_OBJECT_EXPIRED = 6722,
    ERROR_TRANSACTION_RESPONSE_NOT_ENLISTED = 6723,
    ERROR_TRANSACTION_RECORD_TOO_LONG = 6724,
    ERROR_IMPLICIT_TRANSACTION_NOT_SUPPORTED = 6725,
    ERROR_TRANSACTION_INTEGRITY_VIOLATED = 6726,
    ERROR_TRANSACTIONMANAGER_IDENTITY_MISMATCH = 6727,
    ERROR_RM_CANNOT_BE_FROZEN_FOR_SNAPSHOT = 6728,
    ERROR_TRANSACTION_MUST_WRITETHROUGH = 6729,
    ERROR_TRANSACTION_NO_SUPERIOR = 6730,
    ERROR_HEURISTIC_DAMAGE_POSSIBLE = 6731,
    ERROR_TRANSACTIONAL_CONFLICT = 6800,
    ERROR_RM_NOT_ACTIVE = 6801,
    ERROR_RM_METADATA_CORRUPT = 6802,
    ERROR_DIRECTORY_NOT_RM = 6803,
    ERROR_TRANSACTIONS_UNSUPPORTED_REMOTE = 6805,
    ERROR_LOG_RESIZE_INVALID_SIZE = 6806,
    ERROR_OBJECT_NO_LONGER_EXISTS = 6807,
    ERROR_STREAM_MINIVERSION_NOT_FOUND = 6808,
    ERROR_STREAM_MINIVERSION_NOT_VALID = 6809,
    ERROR_MINIVERSION_INACCESSIBLE_FROM_SPECIFIED_TRANSACTION = 6810,
    ERROR_CANT_OPEN_MINIVERSION_WITH_MODIFY_INTENT = 6811,
    ERROR_CANT_CREATE_MORE_STREAM_MINIVERSIONS = 6812,
    ERROR_REMOTE_FILE_VERSION_MISMATCH = 6814,
    ERROR_HANDLE_NO_LONGER_VALID = 6815,
    ERROR_NO_TXF_METADATA = 6816,
    ERROR_LOG_CORRUPTION_DETECTED = 6817,
    ERROR_CANT_RECOVER_WITH_HANDLE_OPEN = 6818,
    ERROR_RM_DISCONNECTED = 6819,
    ERROR_ENLISTMENT_NOT_SUPERIOR = 6820,
    ERROR_RECOVERY_NOT_NEEDED = 6821,
    ERROR_RM_ALREADY_STARTED = 6822,
    ERROR_FILE_IDENTITY_NOT_PERSISTENT = 6823,
    ERROR_CANT_BREAK_TRANSACTIONAL_DEPENDENCY = 6824,
    ERROR_CANT_CROSS_RM_BOUNDARY = 6825,
    ERROR_TXF_DIR_NOT_EMPTY = 6826,
    ERROR_INDOUBT_TRANSACTIONS_EXIST = 6827,
    ERROR_TM_VOLATILE = 6828,
    ERROR_ROLLBACK_TIMER_EXPIRED = 6829,
    ERROR_TXF_ATTRIBUTE_CORRUPT = 6830,
    ERROR_EFS_NOT_ALLOWED_IN_TRANSACTION = 6831,
    ERROR_TRANSACTIONAL_OPEN_NOT_ALLOWED = 6832,
    ERROR_LOG_GROWTH_FAILED = 6833,
    ERROR_TRANSACTED_MAPPING_UNSUPPORTED_REMOTE = 6834,
    ERROR_TXF_METADATA_ALREADY_PRESENT = 6835,
    ERROR_TRANSACTION_SCOPE_CALLBACKS_NOT_SET = 6836,
    ERROR_TRANSACTION_REQUIRED_PROMOTION = 6837,
    ERROR_CANNOT_EXECUTE_FILE_IN_TRANSACTION = 6838,
    ERROR_TRANSACTIONS_NOT_FROZEN = 6839,
    ERROR_TRANSACTION_FREEZE_IN_PROGRESS = 6840,
    ERROR_NOT_SNAPSHOT_VOLUME = 6841,
    ERROR_NO_SAVEPOINT_WITH_OPEN_FILES = 6842,
    ERROR_DATA_LOST_REPAIR = 6843,
    ERROR_SPARSE_NOT_ALLOWED_IN_TRANSACTION = 6844,
    ERROR_TM_IDENTITY_MISMATCH = 6845,
    ERROR_FLOATED_SECTION = 6846,
    ERROR_CANNOT_ACCEPT_TRANSACTED_WORK = 6847,
    ERROR_CANNOT_ABORT_TRANSACTIONS = 6848,
    ERROR_BAD_CLUSTERS = 6849,
    ERROR_COMPRESSION_NOT_ALLOWED_IN_TRANSACTION = 6850,
    ERROR_VOLUME_DIRTY = 6851,
    ERROR_NO_LINK_TRACKING_IN_TRANSACTION = 6852,
    ERROR_OPERATION_NOT_SUPPORTED_IN_TRANSACTION = 6853,
    ERROR_EXPIRED_HANDLE = 6854,
    ERROR_TRANSACTION_NOT_ENLISTED = 6855,
    ERROR_CTX_WINSTATION_NAME_INVALID = 7001,
    ERROR_CTX_INVALID_PD = 7002,
    ERROR_CTX_PD_NOT_FOUND = 7003,
    ERROR_CTX_WD_NOT_FOUND = 7004,
    ERROR_CTX_CANNOT_MAKE_EVENTLOG_ENTRY = 7005,
    ERROR_CTX_SERVICE_NAME_COLLISION = 7006,
    ERROR_CTX_CLOSE_PENDING = 7007,
    ERROR_CTX_NO_OUTBUF = 7008,
    ERROR_CTX_MODEM_INF_NOT_FOUND = 7009,
    ERROR_CTX_INVALID_MODEMNAME = 7010,
    ERROR_CTX_MODEM_RESPONSE_ERROR = 7011,
    ERROR_CTX_MODEM_RESPONSE_TIMEOUT = 7012,
    ERROR_CTX_MODEM_RESPONSE_NO_CARRIER = 7013,
    ERROR_CTX_MODEM_RESPONSE_NO_DIALTONE = 7014,
    ERROR_CTX_MODEM_RESPONSE_BUSY = 7015,
    ERROR_CTX_MODEM_RESPONSE_VOICE = 7016,
    ERROR_CTX_TD_ERROR = 7017,
    ERROR_CTX_WINSTATION_NOT_FOUND = 7022,
    ERROR_CTX_WINSTATION_ALREADY_EXISTS = 7023,
    ERROR_CTX_WINSTATION_BUSY = 7024,
    ERROR_CTX_BAD_VIDEO_MODE = 7025,
    ERROR_CTX_GRAPHICS_INVALID = 7035,
    ERROR_CTX_LOGON_DISABLED = 7037,
    ERROR_CTX_NOT_CONSOLE = 7038,
    ERROR_CTX_CLIENT_QUERY_TIMEOUT = 7040,
    ERROR_CTX_CONSOLE_DISCONNECT = 7041,
    ERROR_CTX_CONSOLE_CONNECT = 7042,
    ERROR_CTX_SHADOW_DENIED = 7044,
    ERROR_CTX_WINSTATION_ACCESS_DENIED = 7045,
    ERROR_CTX_INVALID_WD = 7049,
    ERROR_CTX_SHADOW_INVALID = 7050,
    ERROR_CTX_SHADOW_DISABLED = 7051,
    ERROR_CTX_CLIENT_LICENSE_IN_USE = 7052,
    ERROR_CTX_CLIENT_LICENSE_NOT_SET = 7053,
    ERROR_CTX_LICENSE_NOT_AVAILABLE = 7054,
    ERROR_CTX_LICENSE_CLIENT_INVALID = 7055,
    ERROR_CTX_LICENSE_EXPIRED = 7056,
    ERROR_CTX_SHADOW_NOT_RUNNING = 7057,
    ERROR_CTX_SHADOW_ENDED_BY_MODE_CHANGE = 7058,
    ERROR_ACTIVATION_COUNT_EXCEEDED = 7059,
    ERROR_CTX_WINSTATIONS_DISABLED = 7060,
    ERROR_CTX_ENCRYPTION_LEVEL_REQUIRED = 7061,
    ERROR_CTX_SESSION_IN_USE = 7062,
    ERROR_CTX_NO_FORCE_LOGOFF = 7063,
    ERROR_CTX_ACCOUNT_RESTRICTION = 7064,
    ERROR_RDP_PROTOCOL_ERROR = 7065,
    ERROR_CTX_CDM_CONNECT = 7066,
    ERROR_CTX_CDM_DISCONNECT = 7067,
    ERROR_CTX_SECURITY_LAYER_ERROR = 7068,
    ERROR_TS_INCOMPATIBLE_SESSIONS = 7069,
    ERROR_TS_VIDEO_SUBSYSTEM_ERROR = 7070,
    ERROR_DS_NOT_INSTALLED = 8200,
    ERROR_DS_MEMBERSHIP_EVALUATED_LOCALLY = 8201,
    ERROR_DS_NO_ATTRIBUTE_OR_VALUE = 8202,
    ERROR_DS_INVALID_ATTRIBUTE_SYNTAX = 8203,
    ERROR_DS_ATTRIBUTE_TYPE_UNDEFINED = 8204,
    ERROR_DS_ATTRIBUTE_OR_VALUE_EXISTS = 8205,
    ERROR_DS_BUSY = 8206,
    ERROR_DS_UNAVAILABLE = 8207,
    ERROR_DS_NO_RIDS_ALLOCATED = 8208,
    ERROR_DS_NO_MORE_RIDS = 8209,
    ERROR_DS_INCORRECT_ROLE_OWNER = 8210,
    ERROR_DS_RIDMGR_INIT_ERROR = 8211,
    ERROR_DS_OBJ_CLASS_VIOLATION = 8212,
    ERROR_DS_CANT_ON_NON_LEAF = 8213,
    ERROR_DS_CANT_ON_RDN = 8214,
    ERROR_DS_CANT_MOD_OBJ_CLASS = 8215,
    ERROR_DS_CROSS_DOM_MOVE_ERROR = 8216,
    ERROR_DS_GC_NOT_AVAILABLE = 8217,
    ERROR_SHARED_POLICY = 8218,
    ERROR_POLICY_OBJECT_NOT_FOUND = 8219,
    ERROR_POLICY_ONLY_IN_DS = 8220,
    ERROR_PROMOTION_ACTIVE = 8221,
    ERROR_NO_PROMOTION_ACTIVE = 8222,
    ERROR_DS_OPERATIONS_ERROR = 8224,
    ERROR_DS_PROTOCOL_ERROR = 8225,
    ERROR_DS_TIMELIMIT_EXCEEDED = 8226,
    ERROR_DS_SIZELIMIT_EXCEEDED = 8227,
    ERROR_DS_ADMIN_LIMIT_EXCEEDED = 8228,
    ERROR_DS_COMPARE_FALSE = 8229,
    ERROR_DS_COMPARE_TRUE = 8230,
    ERROR_DS_AUTH_METHOD_NOT_SUPPORTED = 8231,
    ERROR_DS_STRONG_AUTH_REQUIRED = 8232,
    ERROR_DS_INAPPROPRIATE_AUTH = 8233,
    ERROR_DS_AUTH_UNKNOWN = 8234,
    ERROR_DS_REFERRAL = 8235,
    ERROR_DS_UNAVAILABLE_CRIT_EXTENSION = 8236,
    ERROR_DS_CONFIDENTIALITY_REQUIRED = 8237,
    ERROR_DS_INAPPROPRIATE_MATCHING = 8238,
    ERROR_DS_CONSTRAINT_VIOLATION = 8239,
    ERROR_DS_NO_SUCH_OBJECT = 8240,
    ERROR_DS_ALIAS_PROBLEM = 8241,
    ERROR_DS_INVALID_DN_SYNTAX = 8242,
    ERROR_DS_IS_LEAF = 8243,
    ERROR_DS_ALIAS_DEREF_PROBLEM = 8244,
    ERROR_DS_UNWILLING_TO_PERFORM = 8245,
    ERROR_DS_LOOP_DETECT = 8246,
    ERROR_DS_NAMING_VIOLATION = 8247,
    ERROR_DS_OBJECT_RESULTS_TOO_LARGE = 8248,
    ERROR_DS_AFFECTS_MULTIPLE_DSAS = 8249,
    ERROR_DS_SERVER_DOWN = 8250,
    ERROR_DS_LOCAL_ERROR = 8251,
    ERROR_DS_ENCODING_ERROR = 8252,
    ERROR_DS_DECODING_ERROR = 8253,
    ERROR_DS_FILTER_UNKNOWN = 8254,
    ERROR_DS_PARAM_ERROR = 8255,
    ERROR_DS_NOT_SUPPORTED = 8256,
    ERROR_DS_NO_RESULTS_RETURNED = 8257,
    ERROR_DS_CONTROL_NOT_FOUND = 8258,
    ERROR_DS_CLIENT_LOOP = 8259,
    ERROR_DS_REFERRAL_LIMIT_EXCEEDED = 8260,
    ERROR_DS_SORT_CONTROL_MISSING = 8261,
    ERROR_DS_OFFSET_RANGE_ERROR = 8262,
    ERROR_DS_RIDMGR_DISABLED = 8263,
    ERROR_DS_ROOT_MUST_BE_NC = 8301,
    ERROR_DS_ADD_REPLICA_INHIBITED = 8302,
    ERROR_DS_ATT_NOT_DEF_IN_SCHEMA = 8303,
    ERROR_DS_MAX_OBJ_SIZE_EXCEEDED = 8304,
    ERROR_DS_OBJ_STRING_NAME_EXISTS = 8305,
    ERROR_DS_NO_RDN_DEFINED_IN_SCHEMA = 8306,
    ERROR_DS_RDN_DOESNT_MATCH_SCHEMA = 8307,
    ERROR_DS_NO_REQUESTED_ATTS_FOUND = 8308,
    ERROR_DS_USER_BUFFER_TO_SMALL = 8309,
    ERROR_DS_ATT_IS_NOT_ON_OBJ = 8310,
    ERROR_DS_ILLEGAL_MOD_OPERATION = 8311,
    ERROR_DS_OBJ_TOO_LARGE = 8312,
    ERROR_DS_BAD_INSTANCE_TYPE = 8313,
    ERROR_DS_MASTERDSA_REQUIRED = 8314,
    ERROR_DS_OBJECT_CLASS_REQUIRED = 8315,
    ERROR_DS_MISSING_REQUIRED_ATT = 8316,
    ERROR_DS_ATT_NOT_DEF_FOR_CLASS = 8317,
    ERROR_DS_ATT_ALREADY_EXISTS = 8318,
    ERROR_DS_CANT_ADD_ATT_VALUES = 8320,
    ERROR_DS_SINGLE_VALUE_CONSTRAINT = 8321,
    ERROR_DS_RANGE_CONSTRAINT = 8322,
    ERROR_DS_ATT_VAL_ALREADY_EXISTS = 8323,
    ERROR_DS_CANT_REM_MISSING_ATT = 8324,
    ERROR_DS_CANT_REM_MISSING_ATT_VAL = 8325,
    ERROR_DS_ROOT_CANT_BE_SUBREF = 8326,
    ERROR_DS_NO_CHAINING = 8327,
    ERROR_DS_NO_CHAINED_EVAL = 8328,
    ERROR_DS_NO_PARENT_OBJECT = 8329,
    ERROR_DS_PARENT_IS_AN_ALIAS = 8330,
    ERROR_DS_CANT_MIX_MASTER_AND_REPS = 8331,
    ERROR_DS_CHILDREN_EXIST = 8332,
    ERROR_DS_OBJ_NOT_FOUND = 8333,
    ERROR_DS_ALIASED_OBJ_MISSING = 8334,
    ERROR_DS_BAD_NAME_SYNTAX = 8335,
    ERROR_DS_ALIAS_POINTS_TO_ALIAS = 8336,
    ERROR_DS_CANT_DEREF_ALIAS = 8337,
    ERROR_DS_OUT_OF_SCOPE = 8338,
    ERROR_DS_OBJECT_BEING_REMOVED = 8339,
    ERROR_DS_CANT_DELETE_DSA_OBJ = 8340,
    ERROR_DS_GENERIC_ERROR = 8341,
    ERROR_DS_DSA_MUST_BE_INT_MASTER = 8342,
    ERROR_DS_CLASS_NOT_DSA = 8343,
    ERROR_DS_INSUFF_ACCESS_RIGHTS = 8344,
    ERROR_DS_ILLEGAL_SUPERIOR = 8345,
    ERROR_DS_ATTRIBUTE_OWNED_BY_SAM = 8346,
    ERROR_DS_NAME_TOO_MANY_PARTS = 8347,
    ERROR_DS_NAME_TOO_LONG = 8348,
    ERROR_DS_NAME_VALUE_TOO_LONG = 8349,
    ERROR_DS_NAME_UNPARSEABLE = 8350,
    ERROR_DS_NAME_TYPE_UNKNOWN = 8351,
    ERROR_DS_NOT_AN_OBJECT = 8352,
    ERROR_DS_SEC_DESC_TOO_SHORT = 8353,
    ERROR_DS_SEC_DESC_INVALID = 8354,
    ERROR_DS_NO_DELETED_NAME = 8355,
    ERROR_DS_SUBREF_MUST_HAVE_PARENT = 8356,
    ERROR_DS_NCNAME_MUST_BE_NC = 8357,
    ERROR_DS_CANT_ADD_SYSTEM_ONLY = 8358,
    ERROR_DS_CLASS_MUST_BE_CONCRETE = 8359,
    ERROR_DS_INVALID_DMD = 8360,
    ERROR_DS_OBJ_GUID_EXISTS = 8361,
    ERROR_DS_NOT_ON_BACKLINK = 8362,
    ERROR_DS_NO_CROSSREF_FOR_NC = 8363,
    ERROR_DS_SHUTTING_DOWN = 8364,
    ERROR_DS_UNKNOWN_OPERATION = 8365,
    ERROR_DS_INVALID_ROLE_OWNER = 8366,
    ERROR_DS_COULDNT_CONTACT_FSMO = 8367,
    ERROR_DS_CROSS_NC_DN_RENAME = 8368,
    ERROR_DS_CANT_MOD_SYSTEM_ONLY = 8369,
    ERROR_DS_REPLICATOR_ONLY = 8370,
    ERROR_DS_OBJ_CLASS_NOT_DEFINED = 8371,
    ERROR_DS_OBJ_CLASS_NOT_SUBCLASS = 8372,
    ERROR_DS_NAME_REFERENCE_INVALID = 8373,
    ERROR_DS_CROSS_REF_EXISTS = 8374,
    ERROR_DS_CANT_DEL_MASTER_CROSSREF = 8375,
    ERROR_DS_SUBTREE_NOTIFY_NOT_NC_HEAD = 8376,
    ERROR_DS_NOTIFY_FILTER_TOO_COMPLEX = 8377,
    ERROR_DS_DUP_RDN = 8378,
    ERROR_DS_DUP_OID = 8379,
    ERROR_DS_DUP_MAPI_ID = 8380,
    ERROR_DS_DUP_SCHEMA_ID_GUID = 8381,
    ERROR_DS_DUP_LDAP_DISPLAY_NAME = 8382,
    ERROR_DS_SEMANTIC_ATT_TEST = 8383,
    ERROR_DS_SYNTAX_MISMATCH = 8384,
    ERROR_DS_EXISTS_IN_MUST_HAVE = 8385,
    ERROR_DS_EXISTS_IN_MAY_HAVE = 8386,
    ERROR_DS_NONEXISTENT_MAY_HAVE = 8387,
    ERROR_DS_NONEXISTENT_MUST_HAVE = 8388,
    ERROR_DS_AUX_CLS_TEST_FAIL = 8389,
    ERROR_DS_NONEXISTENT_POSS_SUP = 8390,
    ERROR_DS_SUB_CLS_TEST_FAIL = 8391,
    ERROR_DS_BAD_RDN_ATT_ID_SYNTAX = 8392,
    ERROR_DS_EXISTS_IN_AUX_CLS = 8393,
    ERROR_DS_EXISTS_IN_SUB_CLS = 8394,
    ERROR_DS_EXISTS_IN_POSS_SUP = 8395,
    ERROR_DS_RECALCSCHEMA_FAILED = 8396,
    ERROR_DS_TREE_DELETE_NOT_FINISHED = 8397,
    ERROR_DS_CANT_DELETE = 8398,
    ERROR_DS_ATT_SCHEMA_REQ_ID = 8399,
    ERROR_DS_BAD_ATT_SCHEMA_SYNTAX = 8400,
    ERROR_DS_CANT_CACHE_ATT = 8401,
    ERROR_DS_CANT_CACHE_CLASS = 8402,
    ERROR_DS_CANT_REMOVE_ATT_CACHE = 8403,
    ERROR_DS_CANT_REMOVE_CLASS_CACHE = 8404,
    ERROR_DS_CANT_RETRIEVE_DN = 8405,
    ERROR_DS_MISSING_SUPREF = 8406,
    ERROR_DS_CANT_RETRIEVE_INSTANCE = 8407,
    ERROR_DS_CODE_INCONSISTENCY = 8408,
    ERROR_DS_DATABASE_ERROR = 8409,
    ERROR_DS_GOVERNSID_MISSING = 8410,
    ERROR_DS_MISSING_EXPECTED_ATT = 8411,
    ERROR_DS_NCNAME_MISSING_CR_REF = 8412,
    ERROR_DS_SECURITY_CHECKING_ERROR = 8413,
    ERROR_DS_SCHEMA_NOT_LOADED = 8414,
    ERROR_DS_SCHEMA_ALLOC_FAILED = 8415,
    ERROR_DS_ATT_SCHEMA_REQ_SYNTAX = 8416,
    ERROR_DS_GCVERIFY_ERROR = 8417,
    ERROR_DS_DRA_SCHEMA_MISMATCH = 8418,
    ERROR_DS_CANT_FIND_DSA_OBJ = 8419,
    ERROR_DS_CANT_FIND_EXPECTED_NC = 8420,
    ERROR_DS_CANT_FIND_NC_IN_CACHE = 8421,
    ERROR_DS_CANT_RETRIEVE_CHILD = 8422,
    ERROR_DS_SECURITY_ILLEGAL_MODIFY = 8423,
    ERROR_DS_CANT_REPLACE_HIDDEN_REC = 8424,
    ERROR_DS_BAD_HIERARCHY_FILE = 8425,
    ERROR_DS_BUILD_HIERARCHY_TABLE_FAILED = 8426,
    ERROR_DS_CONFIG_PARAM_MISSING = 8427,
    ERROR_DS_COUNTING_AB_INDICES_FAILED = 8428,
    ERROR_DS_HIERARCHY_TABLE_MALLOC_FAILED = 8429,
    ERROR_DS_INTERNAL_FAILURE = 8430,
    ERROR_DS_UNKNOWN_ERROR = 8431,
    ERROR_DS_ROOT_REQUIRES_CLASS_TOP = 8432,
    ERROR_DS_REFUSING_FSMO_ROLES = 8433,
    ERROR_DS_MISSING_FSMO_SETTINGS = 8434,
    ERROR_DS_UNABLE_TO_SURRENDER_ROLES = 8435,
    ERROR_DS_DRA_GENERIC = 8436,
    ERROR_DS_DRA_INVALID_PARAMETER = 8437,
    ERROR_DS_DRA_BUSY = 8438,
    ERROR_DS_DRA_BAD_DN = 8439,
    ERROR_DS_DRA_BAD_NC = 8440,
    ERROR_DS_DRA_DN_EXISTS = 8441,
    ERROR_DS_DRA_INTERNAL_ERROR = 8442,
    ERROR_DS_DRA_INCONSISTENT_DIT = 8443,
    ERROR_DS_DRA_CONNECTION_FAILED = 8444,
    ERROR_DS_DRA_BAD_INSTANCE_TYPE = 8445,
    ERROR_DS_DRA_OUT_OF_MEM = 8446,
    ERROR_DS_DRA_MAIL_PROBLEM = 8447,
    ERROR_DS_DRA_REF_ALREADY_EXISTS = 8448,
    ERROR_DS_DRA_REF_NOT_FOUND = 8449,
    ERROR_DS_DRA_OBJ_IS_REP_SOURCE = 8450,
    ERROR_DS_DRA_DB_ERROR = 8451,
    ERROR_DS_DRA_NO_REPLICA = 8452,
    ERROR_DS_DRA_ACCESS_DENIED = 8453,
    ERROR_DS_DRA_NOT_SUPPORTED = 8454,
    ERROR_DS_DRA_RPC_CANCELLED = 8455,
    ERROR_DS_DRA_SOURCE_DISABLED = 8456,
    ERROR_DS_DRA_SINK_DISABLED = 8457,
    ERROR_DS_DRA_NAME_COLLISION = 8458,
    ERROR_DS_DRA_SOURCE_REINSTALLED = 8459,
    ERROR_DS_DRA_MISSING_PARENT = 8460,
    ERROR_DS_DRA_PREEMPTED = 8461,
    ERROR_DS_DRA_ABANDON_SYNC = 8462,
    ERROR_DS_DRA_SHUTDOWN = 8463,
    ERROR_DS_DRA_INCOMPATIBLE_PARTIAL_SET = 8464,
    ERROR_DS_DRA_SOURCE_IS_PARTIAL_REPLICA = 8465,
    ERROR_DS_DRA_EXTN_CONNECTION_FAILED = 8466,
    ERROR_DS_INSTALL_SCHEMA_MISMATCH = 8467,
    ERROR_DS_DUP_LINK_ID = 8468,
    ERROR_DS_NAME_ERROR_RESOLVING = 8469,
    ERROR_DS_NAME_ERROR_NOT_FOUND = 8470,
    ERROR_DS_NAME_ERROR_NOT_UNIQUE = 8471,
    ERROR_DS_NAME_ERROR_NO_MAPPING = 8472,
    ERROR_DS_NAME_ERROR_DOMAIN_ONLY = 8473,
    ERROR_DS_NAME_ERROR_NO_SYNTACTICAL_MAPPING = 8474,
    ERROR_DS_CONSTRUCTED_ATT_MOD = 8475,
    ERROR_DS_WRONG_OM_OBJ_CLASS = 8476,
    ERROR_DS_DRA_REPL_PENDING = 8477,
    ERROR_DS_DS_REQUIRED = 8478,
    ERROR_DS_INVALID_LDAP_DISPLAY_NAME = 8479,
    ERROR_DS_NON_BASE_SEARCH = 8480,
    ERROR_DS_CANT_RETRIEVE_ATTS = 8481,
    ERROR_DS_BACKLINK_WITHOUT_LINK = 8482,
    ERROR_DS_EPOCH_MISMATCH = 8483,
    ERROR_DS_SRC_NAME_MISMATCH = 8484,
    ERROR_DS_SRC_AND_DST_NC_IDENTICAL = 8485,
    ERROR_DS_DST_NC_MISMATCH = 8486,
    ERROR_DS_NOT_AUTHORITIVE_FOR_DST_NC = 8487,
    ERROR_DS_SRC_GUID_MISMATCH = 8488,
    ERROR_DS_CANT_MOVE_DELETED_OBJECT = 8489,
    ERROR_DS_PDC_OPERATION_IN_PROGRESS = 8490,
    ERROR_DS_CROSS_DOMAIN_CLEANUP_REQD = 8491,
    ERROR_DS_ILLEGAL_XDOM_MOVE_OPERATION = 8492,
    ERROR_DS_CANT_WITH_ACCT_GROUP_MEMBERSHPS = 8493,
    ERROR_DS_NC_MUST_HAVE_NC_PARENT = 8494,
    ERROR_DS_CR_IMPOSSIBLE_TO_VALIDATE = 8495,
    ERROR_DS_DST_DOMAIN_NOT_NATIVE = 8496,
    ERROR_DS_MISSING_INFRASTRUCTURE_CONTAINER = 8497,
    ERROR_DS_CANT_MOVE_ACCOUNT_GROUP = 8498,
    ERROR_DS_CANT_MOVE_RESOURCE_GROUP = 8499,
    ERROR_DS_INVALID_SEARCH_FLAG = 8500,
    ERROR_DS_NO_TREE_DELETE_ABOVE_NC = 8501,
    ERROR_DS_COULDNT_LOCK_TREE_FOR_DELETE = 8502,
    ERROR_DS_COULDNT_IDENTIFY_OBJECTS_FOR_TREE_DELETE = 8503,
    ERROR_DS_SAM_INIT_FAILURE = 8504,
    ERROR_DS_SENSITIVE_GROUP_VIOLATION = 8505,
    ERROR_DS_CANT_MOD_PRIMARYGROUPID = 8506,
    ERROR_DS_ILLEGAL_BASE_SCHEMA_MOD = 8507,
    ERROR_DS_NONSAFE_SCHEMA_CHANGE = 8508,
    ERROR_DS_SCHEMA_UPDATE_DISALLOWED = 8509,
    ERROR_DS_CANT_CREATE_UNDER_SCHEMA = 8510,
    ERROR_DS_INSTALL_NO_SRC_SCH_VERSION = 8511,
    ERROR_DS_INSTALL_NO_SCH_VERSION_IN_INIFILE = 8512,
    ERROR_DS_INVALID_GROUP_TYPE = 8513,
    ERROR_DS_NO_NEST_GLOBALGROUP_IN_MIXEDDOMAIN = 8514,
    ERROR_DS_NO_NEST_LOCALGROUP_IN_MIXEDDOMAIN = 8515,
    ERROR_DS_GLOBAL_CANT_HAVE_LOCAL_MEMBER = 8516,
    ERROR_DS_GLOBAL_CANT_HAVE_UNIVERSAL_MEMBER = 8517,
    ERROR_DS_UNIVERSAL_CANT_HAVE_LOCAL_MEMBER = 8518,
    ERROR_DS_GLOBAL_CANT_HAVE_CROSSDOMAIN_MEMBER = 8519,
    ERROR_DS_LOCAL_CANT_HAVE_CROSSDOMAIN_LOCAL_MEMBER = 8520,
    ERROR_DS_HAVE_PRIMARY_MEMBERS = 8521,
    ERROR_DS_STRING_SD_CONVERSION_FAILED = 8522,
    ERROR_DS_NAMING_MASTER_GC = 8523,
    ERROR_DS_DNS_LOOKUP_FAILURE = 8524,
    ERROR_DS_COULDNT_UPDATE_SPNS = 8525,
    ERROR_DS_CANT_RETRIEVE_SD = 8526,
    ERROR_DS_KEY_NOT_UNIQUE = 8527,
    ERROR_DS_WRONG_LINKED_ATT_SYNTAX = 8528,
    ERROR_DS_SAM_NEED_BOOTKEY_PASSWORD = 8529,
    ERROR_DS_SAM_NEED_BOOTKEY_FLOPPY = 8530,
    ERROR_DS_CANT_START = 8531,
    ERROR_DS_INIT_FAILURE = 8532,
    ERROR_DS_NO_PKT_PRIVACY_ON_CONNECTION = 8533,
    ERROR_DS_SOURCE_DOMAIN_IN_FOREST = 8534,
    ERROR_DS_DESTINATION_DOMAIN_NOT_IN_FOREST = 8535,
    ERROR_DS_DESTINATION_AUDITING_NOT_ENABLED = 8536,
    ERROR_DS_CANT_FIND_DC_FOR_SRC_DOMAIN = 8537,
    ERROR_DS_SRC_OBJ_NOT_GROUP_OR_USER = 8538,
    ERROR_DS_SRC_SID_EXISTS_IN_FOREST = 8539,
    ERROR_DS_SRC_AND_DST_OBJECT_CLASS_MISMATCH = 8540,
    ERROR_SAM_INIT_FAILURE = 8541,
    ERROR_DS_DRA_SCHEMA_INFO_SHIP = 8542,
    ERROR_DS_DRA_SCHEMA_CONFLICT = 8543,
    ERROR_DS_DRA_EARLIER_SCHEMA_CONFLICT = 8544,
    ERROR_DS_DRA_OBJ_NC_MISMATCH = 8545,
    ERROR_DS_NC_STILL_HAS_DSAS = 8546,
    ERROR_DS_GC_REQUIRED = 8547,
    ERROR_DS_LOCAL_MEMBER_OF_LOCAL_ONLY = 8548,
    ERROR_DS_NO_FPO_IN_UNIVERSAL_GROUPS = 8549,
    ERROR_DS_CANT_ADD_TO_GC = 8550,
    ERROR_DS_NO_CHECKPOINT_WITH_PDC = 8551,
    ERROR_DS_SOURCE_AUDITING_NOT_ENABLED = 8552,
    ERROR_DS_CANT_CREATE_IN_NONDOMAIN_NC = 8553,
    ERROR_DS_INVALID_NAME_FOR_SPN = 8554,
    ERROR_DS_FILTER_USES_CONTRUCTED_ATTRS = 8555,
    ERROR_DS_UNICODEPWD_NOT_IN_QUOTES = 8556,
    ERROR_DS_MACHINE_ACCOUNT_QUOTA_EXCEEDED = 8557,
    ERROR_DS_MUST_BE_RUN_ON_DST_DC = 8558,
    ERROR_DS_SRC_DC_MUST_BE_SP4_OR_GREATER = 8559,
    ERROR_DS_CANT_TREE_DELETE_CRITICAL_OBJ = 8560,
    ERROR_DS_INIT_FAILURE_CONSOLE = 8561,
    ERROR_DS_SAM_INIT_FAILURE_CONSOLE = 8562,
    ERROR_DS_FOREST_VERSION_TOO_HIGH = 8563,
    ERROR_DS_DOMAIN_VERSION_TOO_HIGH = 8564,
    ERROR_DS_FOREST_VERSION_TOO_LOW = 8565,
    ERROR_DS_DOMAIN_VERSION_TOO_LOW = 8566,
    ERROR_DS_INCOMPATIBLE_VERSION = 8567,
    ERROR_DS_LOW_DSA_VERSION = 8568,
    ERROR_DS_NO_BEHAVIOR_VERSION_IN_MIXEDDOMAIN = 8569,
    ERROR_DS_NOT_SUPPORTED_SORT_ORDER = 8570,
    ERROR_DS_NAME_NOT_UNIQUE = 8571,
    ERROR_DS_MACHINE_ACCOUNT_CREATED_PRENT4 = 8572,
    ERROR_DS_OUT_OF_VERSION_STORE = 8573,
    ERROR_DS_INCOMPATIBLE_CONTROLS_USED = 8574,
    ERROR_DS_NO_REF_DOMAIN = 8575,
    ERROR_DS_RESERVED_LINK_ID = 8576,
    ERROR_DS_LINK_ID_NOT_AVAILABLE = 8577,
    ERROR_DS_AG_CANT_HAVE_UNIVERSAL_MEMBER = 8578,
    ERROR_DS_MODIFYDN_DISALLOWED_BY_INSTANCE_TYPE = 8579,
    ERROR_DS_NO_OBJECT_MOVE_IN_SCHEMA_NC = 8580,
    ERROR_DS_MODIFYDN_DISALLOWED_BY_FLAG = 8581,
    ERROR_DS_MODIFYDN_WRONG_GRANDPARENT = 8582,
    ERROR_DS_NAME_ERROR_TRUST_REFERRAL = 8583,
    ERROR_NOT_SUPPORTED_ON_STANDARD_SERVER = 8584,
    ERROR_DS_CANT_ACCESS_REMOTE_PART_OF_AD = 8585,
    ERROR_DS_CR_IMPOSSIBLE_TO_VALIDATE_V2 = 8586,
    ERROR_DS_THREAD_LIMIT_EXCEEDED = 8587,
    ERROR_DS_NOT_CLOSEST = 8588,
    ERROR_DS_CANT_DERIVE_SPN_WITHOUT_SERVER_REF = 8589,
    ERROR_DS_SINGLE_USER_MODE_FAILED = 8590,
    ERROR_DS_NTDSCRIPT_SYNTAX_ERROR = 8591,
    ERROR_DS_NTDSCRIPT_PROCESS_ERROR = 8592,
    ERROR_DS_DIFFERENT_REPL_EPOCHS = 8593,
    ERROR_DS_DRS_EXTENSIONS_CHANGED = 8594,
    ERROR_DS_REPLICA_SET_CHANGE_NOT_ALLOWED_ON_DISABLED_CR = 8595,
    ERROR_DS_NO_MSDS_INTID = 8596,
    ERROR_DS_DUP_MSDS_INTID = 8597,
    ERROR_DS_EXISTS_IN_RDNATTID = 8598,
    ERROR_DS_AUTHORIZATION_FAILED = 8599,
    ERROR_DS_INVALID_SCRIPT = 8600,
    ERROR_DS_REMOTE_CROSSREF_OP_FAILED = 8601,
    ERROR_DS_CROSS_REF_BUSY = 8602,
    ERROR_DS_CANT_DERIVE_SPN_FOR_DELETED_DOMAIN = 8603,
    ERROR_DS_CANT_DEMOTE_WITH_WRITEABLE_NC = 8604,
    ERROR_DS_DUPLICATE_ID_FOUND = 8605,
    ERROR_DS_INSUFFICIENT_ATTR_TO_CREATE_OBJECT = 8606,
    ERROR_DS_GROUP_CONVERSION_ERROR = 8607,
    ERROR_DS_CANT_MOVE_APP_BASIC_GROUP = 8608,
    ERROR_DS_CANT_MOVE_APP_QUERY_GROUP = 8609,
    ERROR_DS_ROLE_NOT_VERIFIED = 8610,
    ERROR_DS_WKO_CONTAINER_CANNOT_BE_SPECIAL = 8611,
    ERROR_DS_DOMAIN_RENAME_IN_PROGRESS = 8612,
    ERROR_DS_EXISTING_AD_CHILD_NC = 8613,
    ERROR_DS_REPL_LIFETIME_EXCEEDED = 8614,
    ERROR_DS_DISALLOWED_IN_SYSTEM_CONTAINER = 8615,
    ERROR_DS_LDAP_SEND_QUEUE_FULL = 8616,
    ERROR_DS_DRA_OUT_SCHEDULE_WINDOW = 8617,
    ERROR_DS_POLICY_NOT_KNOWN = 8618,
    ERROR_NO_SITE_SETTINGS_OBJECT = 8619,
    ERROR_NO_SECRETS = 8620,
    ERROR_NO_WRITABLE_DC_FOUND = 8621,
    ERROR_DS_NO_SERVER_OBJECT = 8622,
    ERROR_DS_NO_NTDSA_OBJECT = 8623,
    ERROR_DS_NON_ASQ_SEARCH = 8624,
    ERROR_DS_AUDIT_FAILURE = 8625,
    ERROR_DS_INVALID_SEARCH_FLAG_SUBTREE = 8626,
    ERROR_DS_INVALID_SEARCH_FLAG_TUPLE = 8627,
    ERROR_DS_HIERARCHY_TABLE_TOO_DEEP = 8628,
    ERROR_DS_DRA_CORRUPT_UTD_VECTOR = 8629,
    ERROR_DS_DRA_SECRETS_DENIED = 8630,
    ERROR_DS_RESERVED_MAPI_ID = 8631,
    ERROR_DS_MAPI_ID_NOT_AVAILABLE = 8632,
    ERROR_DS_DRA_MISSING_KRBTGT_SECRET = 8633,
    ERROR_DS_DOMAIN_NAME_EXISTS_IN_FOREST = 8634,
    ERROR_DS_FLAT_NAME_EXISTS_IN_FOREST = 8635,
    ERROR_INVALID_USER_PRINCIPAL_NAME = 8636,
    ERROR_DS_OID_MAPPED_GROUP_CANT_HAVE_MEMBERS = 8637,
    ERROR_DS_OID_NOT_FOUND = 8638,
    ERROR_DS_DRA_RECYCLED_TARGET = 8639,
    ERROR_DS_DISALLOWED_NC_REDIRECT = 8640,
    ERROR_DS_HIGH_ADLDS_FFL = 8641,
    ERROR_DS_HIGH_DSA_VERSION = 8642,
    ERROR_DS_LOW_ADLDS_FFL = 8643,
    ERROR_DOMAIN_SID_SAME_AS_LOCAL_WORKSTATION = 8644,
    ERROR_DS_UNDELETE_SAM_VALIDATION_FAILED = 8645,
    ERROR_INCORRECT_ACCOUNT_TYPE = 8646,
    ERROR_DS_SPN_VALUE_NOT_UNIQUE_IN_FOREST = 8647,
    ERROR_DS_UPN_VALUE_NOT_UNIQUE_IN_FOREST = 8648,
    ERROR_DS_MISSING_FOREST_TRUST = 8649,
    ERROR_DS_VALUE_KEY_NOT_UNIQUE = 8650,
    ERROR_IPSEC_QM_POLICY_EXISTS = 13000,
    ERROR_IPSEC_QM_POLICY_NOT_FOUND = 13001,
    ERROR_IPSEC_QM_POLICY_IN_USE = 13002,
    ERROR_IPSEC_MM_POLICY_EXISTS = 13003,
    ERROR_IPSEC_MM_POLICY_NOT_FOUND = 13004,
    ERROR_IPSEC_MM_POLICY_IN_USE = 13005,
    ERROR_IPSEC_MM_FILTER_EXISTS = 13006,
    ERROR_IPSEC_MM_FILTER_NOT_FOUND = 13007,
    ERROR_IPSEC_TRANSPORT_FILTER_EXISTS = 13008,
    ERROR_IPSEC_TRANSPORT_FILTER_NOT_FOUND = 13009,
    ERROR_IPSEC_MM_AUTH_EXISTS = 13010,
    ERROR_IPSEC_MM_AUTH_NOT_FOUND = 13011,
    ERROR_IPSEC_MM_AUTH_IN_USE = 13012,
    ERROR_IPSEC_DEFAULT_MM_POLICY_NOT_FOUND = 13013,
    ERROR_IPSEC_DEFAULT_MM_AUTH_NOT_FOUND = 13014,
    ERROR_IPSEC_DEFAULT_QM_POLICY_NOT_FOUND = 13015,
    ERROR_IPSEC_TUNNEL_FILTER_EXISTS = 13016,
    ERROR_IPSEC_TUNNEL_FILTER_NOT_FOUND = 13017,
    ERROR_IPSEC_MM_FILTER_PENDING_DELETION = 13018,
    ERROR_IPSEC_TRANSPORT_FILTER_PENDING_DELETION = 13019,
    ERROR_IPSEC_TUNNEL_FILTER_PENDING_DELETION = 13020,
    ERROR_IPSEC_MM_POLICY_PENDING_DELETION = 13021,
    ERROR_IPSEC_MM_AUTH_PENDING_DELETION = 13022,
    ERROR_IPSEC_QM_POLICY_PENDING_DELETION = 13023,
    ERROR_IPSEC_IKE_NEG_STATUS_BEGIN = 13800,
    ERROR_IPSEC_IKE_AUTH_FAIL = 13801,
    ERROR_IPSEC_IKE_ATTRIB_FAIL = 13802,
    ERROR_IPSEC_IKE_NEGOTIATION_PENDING = 13803,
    ERROR_IPSEC_IKE_GENERAL_PROCESSING_ERROR = 13804,
    ERROR_IPSEC_IKE_TIMED_OUT = 13805,
    ERROR_IPSEC_IKE_NO_CERT = 13806,
    ERROR_IPSEC_IKE_SA_DELETED = 13807,
    ERROR_IPSEC_IKE_SA_REAPED = 13808,
    ERROR_IPSEC_IKE_MM_ACQUIRE_DROP = 13809,
    ERROR_IPSEC_IKE_QM_ACQUIRE_DROP = 13810,
    ERROR_IPSEC_IKE_QUEUE_DROP_MM = 13811,
    ERROR_IPSEC_IKE_QUEUE_DROP_NO_MM = 13812,
    ERROR_IPSEC_IKE_DROP_NO_RESPONSE = 13813,
    ERROR_IPSEC_IKE_MM_DELAY_DROP = 13814,
    ERROR_IPSEC_IKE_QM_DELAY_DROP = 13815,
    ERROR_IPSEC_IKE_ERROR = 13816,
    ERROR_IPSEC_IKE_CRL_FAILED = 13817,
    ERROR_IPSEC_IKE_INVALID_KEY_USAGE = 13818,
    ERROR_IPSEC_IKE_INVALID_CERT_TYPE = 13819,
    ERROR_IPSEC_IKE_NO_PRIVATE_KEY = 13820,
    ERROR_IPSEC_IKE_SIMULTANEOUS_REKEY = 13821,
    ERROR_IPSEC_IKE_DH_FAIL = 13822,
    ERROR_IPSEC_IKE_CRITICAL_PAYLOAD_NOT_RECOGNIZED = 13823,
    ERROR_IPSEC_IKE_INVALID_HEADER = 13824,
    ERROR_IPSEC_IKE_NO_POLICY = 13825,
    ERROR_IPSEC_IKE_INVALID_SIGNATURE = 13826,
    ERROR_IPSEC_IKE_KERBEROS_ERROR = 13827,
    ERROR_IPSEC_IKE_NO_PUBLIC_KEY = 13828,
    ERROR_IPSEC_IKE_PROCESS_ERR = 13829,
    ERROR_IPSEC_IKE_PROCESS_ERR_SA = 13830,
    ERROR_IPSEC_IKE_PROCESS_ERR_PROP = 13831,
    ERROR_IPSEC_IKE_PROCESS_ERR_TRANS = 13832,
    ERROR_IPSEC_IKE_PROCESS_ERR_KE = 13833,
    ERROR_IPSEC_IKE_PROCESS_ERR_ID = 13834,
    ERROR_IPSEC_IKE_PROCESS_ERR_CERT = 13835,
    ERROR_IPSEC_IKE_PROCESS_ERR_CERT_REQ = 13836,
    ERROR_IPSEC_IKE_PROCESS_ERR_HASH = 13837,
    ERROR_IPSEC_IKE_PROCESS_ERR_SIG = 13838,
    ERROR_IPSEC_IKE_PROCESS_ERR_NONCE = 13839,
    ERROR_IPSEC_IKE_PROCESS_ERR_NOTIFY = 13840,
    ERROR_IPSEC_IKE_PROCESS_ERR_DELETE = 13841,
    ERROR_IPSEC_IKE_PROCESS_ERR_VENDOR = 13842,
    ERROR_IPSEC_IKE_INVALID_PAYLOAD = 13843,
    ERROR_IPSEC_IKE_LOAD_SOFT_SA = 13844,
    ERROR_IPSEC_IKE_SOFT_SA_TORN_DOWN = 13845,
    ERROR_IPSEC_IKE_INVALID_COOKIE = 13846,
    ERROR_IPSEC_IKE_NO_PEER_CERT = 13847,
    ERROR_IPSEC_IKE_PEER_CRL_FAILED = 13848,
    ERROR_IPSEC_IKE_POLICY_CHANGE = 13849,
    ERROR_IPSEC_IKE_NO_MM_POLICY = 13850,
    ERROR_IPSEC_IKE_NOTCBPRIV = 13851,
    ERROR_IPSEC_IKE_SECLOADFAIL = 13852,
    ERROR_IPSEC_IKE_FAILSSPINIT = 13853,
    ERROR_IPSEC_IKE_FAILQUERYSSP = 13854,
    ERROR_IPSEC_IKE_SRVACQFAIL = 13855,
    ERROR_IPSEC_IKE_SRVQUERYCRED = 13856,
    ERROR_IPSEC_IKE_GETSPIFAIL = 13857,
    ERROR_IPSEC_IKE_INVALID_FILTER = 13858,
    ERROR_IPSEC_IKE_OUT_OF_MEMORY = 13859,
    ERROR_IPSEC_IKE_ADD_UPDATE_KEY_FAILED = 13860,
    ERROR_IPSEC_IKE_INVALID_POLICY = 13861,
    ERROR_IPSEC_IKE_UNKNOWN_DOI = 13862,
    ERROR_IPSEC_IKE_INVALID_SITUATION = 13863,
    ERROR_IPSEC_IKE_DH_FAILURE = 13864,
    ERROR_IPSEC_IKE_INVALID_GROUP = 13865,
    ERROR_IPSEC_IKE_ENCRYPT = 13866,
    ERROR_IPSEC_IKE_DECRYPT = 13867,
    ERROR_IPSEC_IKE_POLICY_MATCH = 13868,
    ERROR_IPSEC_IKE_UNSUPPORTED_ID = 13869,
    ERROR_IPSEC_IKE_INVALID_HASH = 13870,
    ERROR_IPSEC_IKE_INVALID_HASH_ALG = 13871,
    ERROR_IPSEC_IKE_INVALID_HASH_SIZE = 13872,
    ERROR_IPSEC_IKE_INVALID_ENCRYPT_ALG = 13873,
    ERROR_IPSEC_IKE_INVALID_AUTH_ALG = 13874,
    ERROR_IPSEC_IKE_INVALID_SIG = 13875,
    ERROR_IPSEC_IKE_LOAD_FAILED = 13876,
    ERROR_IPSEC_IKE_RPC_DELETE = 13877,
    ERROR_IPSEC_IKE_BENIGN_REINIT = 13878,
    ERROR_IPSEC_IKE_INVALID_RESPONDER_LIFETIME_NOTIFY = 13879,
    ERROR_IPSEC_IKE_INVALID_MAJOR_VERSION = 13880,
    ERROR_IPSEC_IKE_INVALID_CERT_KEYLEN = 13881,
    ERROR_IPSEC_IKE_MM_LIMIT = 13882,
    ERROR_IPSEC_IKE_NEGOTIATION_DISABLED = 13883,
    ERROR_IPSEC_IKE_QM_LIMIT = 13884,
    ERROR_IPSEC_IKE_MM_EXPIRED = 13885,
    ERROR_IPSEC_IKE_PEER_MM_ASSUMED_INVALID = 13886,
    ERROR_IPSEC_IKE_CERT_CHAIN_POLICY_MISMATCH = 13887,
    ERROR_IPSEC_IKE_UNEXPECTED_MESSAGE_ID = 13888,
    ERROR_IPSEC_IKE_INVALID_AUTH_PAYLOAD = 13889,
    ERROR_IPSEC_IKE_DOS_COOKIE_SENT = 13890,
    ERROR_IPSEC_IKE_SHUTTING_DOWN = 13891,
    ERROR_IPSEC_IKE_CGA_AUTH_FAILED = 13892,
    ERROR_IPSEC_IKE_PROCESS_ERR_NATOA = 13893,
    ERROR_IPSEC_IKE_INVALID_MM_FOR_QM = 13894,
    ERROR_IPSEC_IKE_QM_EXPIRED = 13895,
    ERROR_IPSEC_IKE_TOO_MANY_FILTERS = 13896,
    ERROR_IPSEC_IKE_NEG_STATUS_END = 13897,
    ERROR_IPSEC_IKE_KILL_DUMMY_NAP_TUNNEL = 13898,
    ERROR_IPSEC_IKE_INNER_IP_ASSIGNMENT_FAILURE = 13899,
    ERROR_IPSEC_IKE_REQUIRE_CP_PAYLOAD_MISSING = 13900,
    ERROR_IPSEC_KEY_MODULE_IMPERSONATION_NEGOTIATION_PENDING = 13901,
    ERROR_IPSEC_IKE_COEXISTENCE_SUPPRESS = 13902,
    ERROR_IPSEC_IKE_RATELIMIT_DROP = 13903,
    ERROR_IPSEC_IKE_PEER_DOESNT_SUPPORT_MOBIKE = 13904,
    ERROR_IPSEC_IKE_AUTHORIZATION_FAILURE = 13905,
    ERROR_IPSEC_IKE_STRONG_CRED_AUTHORIZATION_FAILURE = 13906,
    ERROR_IPSEC_IKE_AUTHORIZATION_FAILURE_WITH_OPTIONAL_RETRY = 13907,
    ERROR_IPSEC_IKE_STRONG_CRED_AUTHORIZATION_AND_CERTMAP_FAILURE = 13908,
    ERROR_IPSEC_IKE_NEG_STATUS_EXTENDED_END = 13909,
    ERROR_IPSEC_BAD_SPI = 13910,
    ERROR_IPSEC_SA_LIFETIME_EXPIRED = 13911,
    ERROR_IPSEC_WRONG_SA = 13912,
    ERROR_IPSEC_REPLAY_CHECK_FAILED = 13913,
    ERROR_IPSEC_INVALID_PACKET = 13914,
    ERROR_IPSEC_INTEGRITY_CHECK_FAILED = 13915,
    ERROR_IPSEC_CLEAR_TEXT_DROP = 13916,
    ERROR_IPSEC_AUTH_FIREWALL_DROP = 13917,
    ERROR_IPSEC_THROTTLE_DROP = 13918,
    ERROR_IPSEC_DOSP_BLOCK = 13925,
    ERROR_IPSEC_DOSP_RECEIVED_MULTICAST = 13926,
    ERROR_IPSEC_DOSP_INVALID_PACKET = 13927,
    ERROR_IPSEC_DOSP_STATE_LOOKUP_FAILED = 13928,
    ERROR_IPSEC_DOSP_MAX_ENTRIES = 13929,
    ERROR_IPSEC_DOSP_KEYMOD_NOT_ALLOWED = 13930,
    ERROR_IPSEC_DOSP_NOT_INSTALLED = 13931,
    ERROR_IPSEC_DOSP_MAX_PER_IP_RATELIMIT_QUEUES = 13932,
    ERROR_SXS_SECTION_NOT_FOUND = 14000,
    ERROR_SXS_CANT_GEN_ACTCTX = 14001,
    ERROR_SXS_INVALID_ACTCTXDATA_FORMAT = 14002,
    ERROR_SXS_ASSEMBLY_NOT_FOUND = 14003,
    ERROR_SXS_MANIFEST_FORMAT_ERROR = 14004,
    ERROR_SXS_MANIFEST_PARSE_ERROR = 14005,
    ERROR_SXS_ACTIVATION_CONTEXT_DISABLED = 14006,
    ERROR_SXS_KEY_NOT_FOUND = 14007,
    ERROR_SXS_VERSION_CONFLICT = 14008,
    ERROR_SXS_WRONG_SECTION_TYPE = 14009,
    ERROR_SXS_THREAD_QUERIES_DISABLED = 14010,
    ERROR_SXS_PROCESS_DEFAULT_ALREADY_SET = 14011,
    ERROR_SXS_UNKNOWN_ENCODING_GROUP = 14012,
    ERROR_SXS_UNKNOWN_ENCODING = 14013,
    ERROR_SXS_INVALID_XML_NAMESPACE_URI = 14014,
    ERROR_SXS_ROOT_MANIFEST_DEPENDENCY_NOT_INSTALLED = 14015,
    ERROR_SXS_LEAF_MANIFEST_DEPENDENCY_NOT_INSTALLED = 14016,
    ERROR_SXS_INVALID_ASSEMBLY_IDENTITY_ATTRIBUTE = 14017,
    ERROR_SXS_MANIFEST_MISSING_REQUIRED_DEFAULT_NAMESPACE = 14018,
    ERROR_SXS_MANIFEST_INVALID_REQUIRED_DEFAULT_NAMESPACE = 14019,
    ERROR_SXS_PRIVATE_MANIFEST_CROSS_PATH_WITH_REPARSE_POINT = 14020,
    ERROR_SXS_DUPLICATE_DLL_NAME = 14021,
    ERROR_SXS_DUPLICATE_WINDOWCLASS_NAME = 14022,
    ERROR_SXS_DUPLICATE_CLSID = 14023,
    ERROR_SXS_DUPLICATE_IID = 14024,
    ERROR_SXS_DUPLICATE_TLBID = 14025,
    ERROR_SXS_DUPLICATE_PROGID = 14026,
    ERROR_SXS_DUPLICATE_ASSEMBLY_NAME = 14027,
    ERROR_SXS_FILE_HASH_MISMATCH = 14028,
    ERROR_SXS_POLICY_PARSE_ERROR = 14029,
    ERROR_SXS_XML_E_MISSINGQUOTE = 14030,
    ERROR_SXS_XML_E_COMMENTSYNTAX = 14031,
    ERROR_SXS_XML_E_BADSTARTNAMECHAR = 14032,
    ERROR_SXS_XML_E_BADNAMECHAR = 14033,
    ERROR_SXS_XML_E_BADCHARINSTRING = 14034,
    ERROR_SXS_XML_E_XMLDECLSYNTAX = 14035,
    ERROR_SXS_XML_E_BADCHARDATA = 14036,
    ERROR_SXS_XML_E_MISSINGWHITESPACE = 14037,
    ERROR_SXS_XML_E_EXPECTINGTAGEND = 14038,
    ERROR_SXS_XML_E_MISSINGSEMICOLON = 14039,
    ERROR_SXS_XML_E_UNBALANCEDPAREN = 14040,
    ERROR_SXS_XML_E_INTERNALERROR = 14041,
    ERROR_SXS_XML_E_UNEXPECTED_WHITESPACE = 14042,
    ERROR_SXS_XML_E_INCOMPLETE_ENCODING = 14043,
    ERROR_SXS_XML_E_MISSING_PAREN = 14044,
    ERROR_SXS_XML_E_EXPECTINGCLOSEQUOTE = 14045,
    ERROR_SXS_XML_E_MULTIPLE_COLONS = 14046,
    ERROR_SXS_XML_E_INVALID_DECIMAL = 14047,
    ERROR_SXS_XML_E_INVALID_HEXIDECIMAL = 14048,
    ERROR_SXS_XML_E_INVALID_UNICODE = 14049,
    ERROR_SXS_XML_E_WHITESPACEORQUESTIONMARK = 14050,
    ERROR_SXS_XML_E_UNEXPECTEDENDTAG = 14051,
    ERROR_SXS_XML_E_UNCLOSEDTAG = 14052,
    ERROR_SXS_XML_E_DUPLICATEATTRIBUTE = 14053,
    ERROR_SXS_XML_E_MULTIPLEROOTS = 14054,
    ERROR_SXS_XML_E_INVALIDATROOTLEVEL = 14055,
    ERROR_SXS_XML_E_BADXMLDECL = 14056,
    ERROR_SXS_XML_E_MISSINGROOT = 14057,
    ERROR_SXS_XML_E_UNEXPECTEDEOF = 14058,
    ERROR_SXS_XML_E_BADPEREFINSUBSET = 14059,
    ERROR_SXS_XML_E_UNCLOSEDSTARTTAG = 14060,
    ERROR_SXS_XML_E_UNCLOSEDENDTAG = 14061,
    ERROR_SXS_XML_E_UNCLOSEDSTRING = 14062,
    ERROR_SXS_XML_E_UNCLOSEDCOMMENT = 14063,
    ERROR_SXS_XML_E_UNCLOSEDDECL = 14064,
    ERROR_SXS_XML_E_UNCLOSEDCDATA = 14065,
    ERROR_SXS_XML_E_RESERVEDNAMESPACE = 14066,
    ERROR_SXS_XML_E_INVALIDENCODING = 14067,
    ERROR_SXS_XML_E_INVALIDSWITCH = 14068,
    ERROR_SXS_XML_E_BADXMLCASE = 14069,
    ERROR_SXS_XML_E_INVALID_STANDALONE = 14070,
    ERROR_SXS_XML_E_UNEXPECTED_STANDALONE = 14071,
    ERROR_SXS_XML_E_INVALID_VERSION = 14072,
    ERROR_SXS_XML_E_MISSINGEQUALS = 14073,
    ERROR_SXS_PROTECTION_RECOVERY_FAILED = 14074,
    ERROR_SXS_PROTECTION_PUBLIC_KEY_TOO_SHORT = 14075,
    ERROR_SXS_PROTECTION_CATALOG_NOT_VALID = 14076,
    ERROR_SXS_UNTRANSLATABLE_HRESULT = 14077,
    ERROR_SXS_PROTECTION_CATALOG_FILE_MISSING = 14078,
    ERROR_SXS_MISSING_ASSEMBLY_IDENTITY_ATTRIBUTE = 14079,
    ERROR_SXS_INVALID_ASSEMBLY_IDENTITY_ATTRIBUTE_NAME = 14080,
    ERROR_SXS_ASSEMBLY_MISSING = 14081,
    ERROR_SXS_CORRUPT_ACTIVATION_STACK = 14082,
    ERROR_SXS_CORRUPTION = 14083,
    ERROR_SXS_EARLY_DEACTIVATION = 14084,
    ERROR_SXS_INVALID_DEACTIVATION = 14085,
    ERROR_SXS_MULTIPLE_DEACTIVATION = 14086,
    ERROR_SXS_PROCESS_TERMINATION_REQUESTED = 14087,
    ERROR_SXS_RELEASE_ACTIVATION_CONTEXT = 14088,
    ERROR_SXS_SYSTEM_DEFAULT_ACTIVATION_CONTEXT_EMPTY = 14089,
    ERROR_SXS_INVALID_IDENTITY_ATTRIBUTE_VALUE = 14090,
    ERROR_SXS_INVALID_IDENTITY_ATTRIBUTE_NAME = 14091,
    ERROR_SXS_IDENTITY_DUPLICATE_ATTRIBUTE = 14092,
    ERROR_SXS_IDENTITY_PARSE_ERROR = 14093,
    ERROR_MALFORMED_SUBSTITUTION_STRING = 14094,
    ERROR_SXS_INCORRECT_PUBLIC_KEY_TOKEN = 14095,
    ERROR_UNMAPPED_SUBSTITUTION_STRING = 14096,
    ERROR_SXS_ASSEMBLY_NOT_LOCKED = 14097,
    ERROR_SXS_COMPONENT_STORE_CORRUPT = 14098,
    ERROR_ADVANCED_INSTALLER_FAILED = 14099,
    ERROR_XML_ENCODING_MISMATCH = 14100,
    ERROR_SXS_MANIFEST_IDENTITY_SAME_BUT_CONTENTS_DIFFERENT = 14101,
    ERROR_SXS_IDENTITIES_DIFFERENT = 14102,
    ERROR_SXS_ASSEMBLY_IS_NOT_A_DEPLOYMENT = 14103,
    ERROR_SXS_FILE_NOT_PART_OF_ASSEMBLY = 14104,
    ERROR_SXS_MANIFEST_TOO_BIG = 14105,
    ERROR_SXS_SETTING_NOT_REGISTERED = 14106,
    ERROR_SXS_TRANSACTION_CLOSURE_INCOMPLETE = 14107,
    ERROR_SMI_PRIMITIVE_INSTALLER_FAILED = 14108,
    ERROR_GENERIC_COMMAND_FAILED = 14109,
    ERROR_SXS_FILE_HASH_MISSING = 14110,
    ERROR_SXS_DUPLICATE_ACTIVATABLE_CLASS = 14111,
    ERROR_EVT_INVALID_CHANNEL_PATH = 15000,
    ERROR_EVT_INVALID_QUERY = 15001,
    ERROR_EVT_PUBLISHER_METADATA_NOT_FOUND = 15002,
    ERROR_EVT_EVENT_TEMPLATE_NOT_FOUND = 15003,
    ERROR_EVT_INVALID_PUBLISHER_NAME = 15004,
    ERROR_EVT_INVALID_EVENT_DATA = 15005,
    ERROR_EVT_CHANNEL_NOT_FOUND = 15007,
    ERROR_EVT_MALFORMED_XML_TEXT = 15008,
    ERROR_EVT_SUBSCRIPTION_TO_DIRECT_CHANNEL = 15009,
    ERROR_EVT_CONFIGURATION_ERROR = 15010,
    ERROR_EVT_QUERY_RESULT_STALE = 15011,
    ERROR_EVT_QUERY_RESULT_INVALID_POSITION = 15012,
    ERROR_EVT_NON_VALIDATING_MSXML = 15013,
    ERROR_EVT_FILTER_ALREADYSCOPED = 15014,
    ERROR_EVT_FILTER_NOTELTSET = 15015,
    ERROR_EVT_FILTER_INVARG = 15016,
    ERROR_EVT_FILTER_INVTEST = 15017,
    ERROR_EVT_FILTER_INVTYPE = 15018,
    ERROR_EVT_FILTER_PARSEERR = 15019,
    ERROR_EVT_FILTER_UNSUPPORTEDOP = 15020,
    ERROR_EVT_FILTER_UNEXPECTEDTOKEN = 15021,
    ERROR_EVT_INVALID_OPERATION_OVER_ENABLED_DIRECT_CHANNEL = 15022,
    ERROR_EVT_INVALID_CHANNEL_PROPERTY_VALUE = 15023,
    ERROR_EVT_INVALID_PUBLISHER_PROPERTY_VALUE = 15024,
    ERROR_EVT_CHANNEL_CANNOT_ACTIVATE = 15025,
    ERROR_EVT_FILTER_TOO_COMPLEX = 15026,
    ERROR_EVT_MESSAGE_NOT_FOUND = 15027,
    ERROR_EVT_MESSAGE_ID_NOT_FOUND = 15028,
    ERROR_EVT_UNRESOLVED_VALUE_INSERT = 15029,
    ERROR_EVT_UNRESOLVED_PARAMETER_INSERT = 15030,
    ERROR_EVT_MAX_INSERTS_REACHED = 15031,
    ERROR_EVT_EVENT_DEFINITION_NOT_FOUND = 15032,
    ERROR_EVT_MESSAGE_LOCALE_NOT_FOUND = 15033,
    ERROR_EVT_VERSION_TOO_OLD = 15034,
    ERROR_EVT_VERSION_TOO_NEW = 15035,
    ERROR_EVT_CANNOT_OPEN_CHANNEL_OF_QUERY = 15036,
    ERROR_EVT_PUBLISHER_DISABLED = 15037,
    ERROR_EVT_FILTER_OUT_OF_RANGE = 15038,
    ERROR_EC_SUBSCRIPTION_CANNOT_ACTIVATE = 15080,
    ERROR_EC_LOG_DISABLED = 15081,
    ERROR_EC_CIRCULAR_FORWARDING = 15082,
    ERROR_EC_CREDSTORE_FULL = 15083,
    ERROR_EC_CRED_NOT_FOUND = 15084,
    ERROR_EC_NO_ACTIVE_CHANNEL = 15085,
    ERROR_MUI_FILE_NOT_FOUND = 15100,
    ERROR_MUI_INVALID_FILE = 15101,
    ERROR_MUI_INVALID_RC_CONFIG = 15102,
    ERROR_MUI_INVALID_LOCALE_NAME = 15103,
    ERROR_MUI_INVALID_ULTIMATEFALLBACK_NAME = 15104,
    ERROR_MUI_FILE_NOT_LOADED = 15105,
    ERROR_RESOURCE_ENUM_USER_STOP = 15106,
    ERROR_MUI_INTLSETTINGS_UILANG_NOT_INSTALLED = 15107,
    ERROR_MUI_INTLSETTINGS_INVALID_LOCALE_NAME = 15108,
    ERROR_MRM_RUNTIME_NO_DEFAULT_OR_NEUTRAL_RESOURCE = 15110,
    ERROR_MRM_INVALID_PRICONFIG = 15111,
    ERROR_MRM_INVALID_FILE_TYPE = 15112,
    ERROR_MRM_UNKNOWN_QUALIFIER = 15113,
    ERROR_MRM_INVALID_QUALIFIER_VALUE = 15114,
    ERROR_MRM_NO_CANDIDATE = 15115,
    ERROR_MRM_NO_MATCH_OR_DEFAULT_CANDIDATE = 15116,
    ERROR_MRM_RESOURCE_TYPE_MISMATCH = 15117,
    ERROR_MRM_DUPLICATE_MAP_NAME = 15118,
    ERROR_MRM_DUPLICATE_ENTRY = 15119,
    ERROR_MRM_INVALID_RESOURCE_IDENTIFIER = 15120,
    ERROR_MRM_FILEPATH_TOO_LONG = 15121,
    ERROR_MRM_UNSUPPORTED_DIRECTORY_TYPE = 15122,
    ERROR_MRM_INVALID_PRI_FILE = 15126,
    ERROR_MRM_NAMED_RESOURCE_NOT_FOUND = 15127,
    ERROR_MRM_MAP_NOT_FOUND = 15135,
    ERROR_MRM_UNSUPPORTED_PROFILE_TYPE = 15136,
    ERROR_MRM_INVALID_QUALIFIER_OPERATOR = 15137,
    ERROR_MRM_INDETERMINATE_QUALIFIER_VALUE = 15138,
    ERROR_MRM_AUTOMERGE_ENABLED = 15139,
    ERROR_MRM_TOO_MANY_RESOURCES = 15140,
    ERROR_MRM_UNSUPPORTED_FILE_TYPE_FOR_MERGE = 15141,
    ERROR_MRM_UNSUPPORTED_FILE_TYPE_FOR_LOAD_UNLOAD_PRI_FILE = 15142,
    ERROR_MRM_NO_CURRENT_VIEW_ON_THREAD = 15143,
    ERROR_DIFFERENT_PROFILE_RESOURCE_MANAGER_EXIST = 15144,
    ERROR_OPERATION_NOT_ALLOWED_FROM_SYSTEM_COMPONENT = 15145,
    ERROR_MRM_DIRECT_REF_TO_NON_DEFAULT_RESOURCE = 15146,
    ERROR_MRM_GENERATION_COUNT_MISMATCH = 15147,
    ERROR_PRI_MERGE_VERSION_MISMATCH = 15148,
    ERROR_PRI_MERGE_MISSING_SCHEMA = 15149,
    ERROR_PRI_MERGE_LOAD_FILE_FAILED = 15150,
    ERROR_PRI_MERGE_ADD_FILE_FAILED = 15151,
    ERROR_PRI_MERGE_WRITE_FILE_FAILED = 15152,
    ERROR_PRI_MERGE_MULTIPLE_PACKAGE_FAMILIES_NOT_ALLOWED = 15153,
    ERROR_PRI_MERGE_MULTIPLE_MAIN_PACKAGES_NOT_ALLOWED = 15154,
    ERROR_PRI_MERGE_BUNDLE_PACKAGES_NOT_ALLOWED = 15155,
    ERROR_PRI_MERGE_MAIN_PACKAGE_REQUIRED = 15156,
    ERROR_PRI_MERGE_RESOURCE_PACKAGE_REQUIRED = 15157,
    ERROR_PRI_MERGE_INVALID_FILE_NAME = 15158,
    ERROR_MRM_PACKAGE_NOT_FOUND = 15159,
    ERROR_MRM_MISSING_DEFAULT_LANGUAGE = 15160,
    ERROR_MCA_INVALID_CAPABILITIES_STRING = 15200,
    ERROR_MCA_INVALID_VCP_VERSION = 15201,
    ERROR_MCA_MONITOR_VIOLATES_MCCS_SPECIFICATION = 15202,
    ERROR_MCA_MCCS_VERSION_MISMATCH = 15203,
    ERROR_MCA_UNSUPPORTED_MCCS_VERSION = 15204,
    ERROR_MCA_INTERNAL_ERROR = 15205,
    ERROR_MCA_INVALID_TECHNOLOGY_TYPE_RETURNED = 15206,
    ERROR_MCA_UNSUPPORTED_COLOR_TEMPERATURE = 15207,
    ERROR_AMBIGUOUS_SYSTEM_DEVICE = 15250,
    ERROR_SYSTEM_DEVICE_NOT_FOUND = 15299,
    ERROR_HASH_NOT_SUPPORTED = 15300,
    ERROR_HASH_NOT_PRESENT = 15301,
    ERROR_SECONDARY_IC_PROVIDER_NOT_REGISTERED = 15321,
    ERROR_GPIO_CLIENT_INFORMATION_INVALID = 15322,
    ERROR_GPIO_VERSION_NOT_SUPPORTED = 15323,
    ERROR_GPIO_INVALID_REGISTRATION_PACKET = 15324,
    ERROR_GPIO_OPERATION_DENIED = 15325,
    ERROR_GPIO_INCOMPATIBLE_CONNECT_MODE = 15326,
    ERROR_GPIO_INTERRUPT_ALREADY_UNMASKED = 15327,
    ERROR_CANNOT_SWITCH_RUNLEVEL = 15400,
    ERROR_INVALID_RUNLEVEL_SETTING = 15401,
    ERROR_RUNLEVEL_SWITCH_TIMEOUT = 15402,
    ERROR_RUNLEVEL_SWITCH_AGENT_TIMEOUT = 15403,
    ERROR_RUNLEVEL_SWITCH_IN_PROGRESS = 15404,
    ERROR_SERVICES_FAILED_AUTOSTART = 15405,
    ERROR_COM_TASK_STOP_PENDING = 15501,
    ERROR_INSTALL_OPEN_PACKAGE_FAILED = 15600,
    ERROR_INSTALL_PACKAGE_NOT_FOUND = 15601,
    ERROR_INSTALL_INVALID_PACKAGE = 15602,
    ERROR_INSTALL_RESOLVE_DEPENDENCY_FAILED = 15603,
    ERROR_INSTALL_OUT_OF_DISK_SPACE = 15604,
    ERROR_INSTALL_NETWORK_FAILURE = 15605,
    ERROR_INSTALL_REGISTRATION_FAILURE = 15606,
    ERROR_INSTALL_DEREGISTRATION_FAILURE = 15607,
    ERROR_INSTALL_CANCEL = 15608,
    ERROR_INSTALL_FAILED = 15609,
    ERROR_REMOVE_FAILED = 15610,
    ERROR_PACKAGE_ALREADY_EXISTS = 15611,
    ERROR_NEEDS_REMEDIATION = 15612,
    ERROR_INSTALL_PREREQUISITE_FAILED = 15613,
    ERROR_PACKAGE_REPOSITORY_CORRUPTED = 15614,
    ERROR_INSTALL_POLICY_FAILURE = 15615,
    ERROR_PACKAGE_UPDATING = 15616,
    ERROR_DEPLOYMENT_BLOCKED_BY_POLICY = 15617,
    ERROR_PACKAGES_IN_USE = 15618,
    ERROR_RECOVERY_FILE_CORRUPT = 15619,
    ERROR_INVALID_STAGED_SIGNATURE = 15620,
    ERROR_DELETING_EXISTING_APPLICATIONDATA_STORE_FAILED = 15621,
    ERROR_INSTALL_PACKAGE_DOWNGRADE = 15622,
    ERROR_SYSTEM_NEEDS_REMEDIATION = 15623,
    ERROR_APPX_INTEGRITY_FAILURE_CLR_NGEN = 15624,
    ERROR_RESILIENCY_FILE_CORRUPT = 15625,
    ERROR_INSTALL_FIREWALL_SERVICE_NOT_RUNNING = 15626,
    ERROR_PACKAGE_MOVE_FAILED = 15627,
    ERROR_INSTALL_VOLUME_NOT_EMPTY = 15628,
    ERROR_INSTALL_VOLUME_OFFLINE = 15629,
    ERROR_INSTALL_VOLUME_CORRUPT = 15630,
    ERROR_NEEDS_REGISTRATION = 15631,
    ERROR_INSTALL_WRONG_PROCESSOR_ARCHITECTURE = 15632,
    ERROR_DEV_SIDELOAD_LIMIT_EXCEEDED = 15633,
    ERROR_INSTALL_OPTIONAL_PACKAGE_REQUIRES_MAIN_PACKAGE = 15634,
    ERROR_PACKAGE_NOT_SUPPORTED_ON_FILESYSTEM = 15635,
    ERROR_PACKAGE_MOVE_BLOCKED_BY_STREAMING = 15636,
    ERROR_INSTALL_OPTIONAL_PACKAGE_APPLICATIONID_NOT_UNIQUE = 15637,
    ERROR_PACKAGE_STAGING_ONHOLD = 15638,
    ERROR_INSTALL_INVALID_RELATED_SET_UPDATE = 15639,
    ERROR_INSTALL_OPTIONAL_PACKAGE_REQUIRES_MAIN_PACKAGE_FULLTRUST_CAPABILITY = 15640,
    ERROR_DEPLOYMENT_BLOCKED_BY_USER_LOG_OFF = 15641,
    ERROR_PROVISION_OPTIONAL_PACKAGE_REQUIRES_MAIN_PACKAGE_PROVISIONED = 15642,
    ERROR_PACKAGES_REPUTATION_CHECK_FAILED = 15643,
    ERROR_PACKAGES_REPUTATION_CHECK_TIMEDOUT = 15644,
    ERROR_DEPLOYMENT_OPTION_NOT_SUPPORTED = 15645,
    ERROR_APPINSTALLER_ACTIVATION_BLOCKED = 15646,
    ERROR_REGISTRATION_FROM_REMOTE_DRIVE_NOT_SUPPORTED = 15647,
    ERROR_APPX_RAW_DATA_WRITE_FAILED = 15648,
    ERROR_DEPLOYMENT_BLOCKED_BY_VOLUME_POLICY_PACKAGE = 15649,
    ERROR_DEPLOYMENT_BLOCKED_BY_VOLUME_POLICY_MACHINE = 15650,
    ERROR_DEPLOYMENT_BLOCKED_BY_PROFILE_POLICY = 15651,
    ERROR_DEPLOYMENT_FAILED_CONFLICTING_MUTABLE_PACKAGE_DIRECTORY = 15652,
    ERROR_SINGLETON_RESOURCE_INSTALLED_IN_ACTIVE_USER = 15653,
    ERROR_DIFFERENT_VERSION_OF_PACKAGED_SERVICE_INSTALLED = 15654,
    ERROR_SERVICE_EXISTS_AS_NON_PACKAGED_SERVICE = 15655,
    ERROR_PACKAGED_SERVICE_REQUIRES_ADMIN_PRIVILEGES = 15656,
    ERROR_REDIRECTION_TO_DEFAULT_ACCOUNT_NOT_ALLOWED = 15657,
    ERROR_PACKAGE_LACKS_CAPABILITY_TO_DEPLOY_ON_HOST = 15658,
    ERROR_UNSIGNED_PACKAGE_INVALID_CONTENT = 15659,
    ERROR_UNSIGNED_PACKAGE_INVALID_PUBLISHER_NAMESPACE = 15660,
    ERROR_SIGNED_PACKAGE_INVALID_PUBLISHER_NAMESPACE = 15661,
    ERROR_PACKAGE_EXTERNAL_LOCATION_NOT_ALLOWED = 15662,
    ERROR_INSTALL_FULLTRUST_HOSTRUNTIME_REQUIRES_MAIN_PACKAGE_FULLTRUST_CAPABILITY = 15663,
    ERROR_STATE_LOAD_STORE_FAILED = 15800,
    ERROR_STATE_GET_VERSION_FAILED = 15801,
    ERROR_STATE_SET_VERSION_FAILED = 15802,
    ERROR_STATE_STRUCTURED_RESET_FAILED = 15803,
    ERROR_STATE_OPEN_CONTAINER_FAILED = 15804,
    ERROR_STATE_CREATE_CONTAINER_FAILED = 15805,
    ERROR_STATE_DELETE_CONTAINER_FAILED = 15806,
    ERROR_STATE_READ_SETTING_FAILED = 15807,
    ERROR_STATE_WRITE_SETTING_FAILED = 15808,
    ERROR_STATE_DELETE_SETTING_FAILED = 15809,
    ERROR_STATE_QUERY_SETTING_FAILED = 15810,
    ERROR_STATE_READ_COMPOSITE_SETTING_FAILED = 15811,
    ERROR_STATE_WRITE_COMPOSITE_SETTING_FAILED = 15812,
    ERROR_STATE_ENUMERATE_CONTAINER_FAILED = 15813,
    ERROR_STATE_ENUMERATE_SETTINGS_FAILED = 15814,
    ERROR_STATE_COMPOSITE_SETTING_VALUE_SIZE_LIMIT_EXCEEDED = 15815,
    ERROR_STATE_SETTING_VALUE_SIZE_LIMIT_EXCEEDED = 15816,
    ERROR_STATE_SETTING_NAME_SIZE_LIMIT_EXCEEDED = 15817,
    ERROR_STATE_CONTAINER_NAME_SIZE_LIMIT_EXCEEDED = 15818,
    ERROR_API_UNAVAILABLE = 15841,
}

/// Values for [`MINIDUMP_EXCEPTION::exception_code`] for crashes on Windows and also
/// for sub-codes and last reported errors
///
/// The values were generated from from ntstatus.h in the Windows 10 SDK
/// (version 10.0.19041.0) using the following script:
/// ```sh
/// egrep '#define [A-Z_0-9]+\s+\(\(NTSTATUS\)0x[48C][0-9A-F]+L\)' ntstatus.h \
///   | tr -d '\r' \
///   | sed -r 's@#define ([A-Z_0-9]+)\s+\(\(NTSTATUS\)(0x[48C][0-9A-F]+)L\).*@\2 \1@' \
///   | sort \
///   | sed -r 's@(0x[48C][0-9A-F]+) ([A-Z_0-9]+)@    \2 = \L\1,@'
/// ```
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum NtStatusWindows {
    STATUS_OBJECT_NAME_EXISTS = 0x40000000u32,
    STATUS_THREAD_WAS_SUSPENDED = 0x40000001,
    STATUS_WORKING_SET_LIMIT_RANGE = 0x40000002,
    STATUS_IMAGE_NOT_AT_BASE = 0x40000003,
    STATUS_RXACT_STATE_CREATED = 0x40000004,
    STATUS_SEGMENT_NOTIFICATION = 0x40000005,
    STATUS_LOCAL_USER_SESSION_KEY = 0x40000006,
    STATUS_BAD_CURRENT_DIRECTORY = 0x40000007,
    STATUS_SERIAL_MORE_WRITES = 0x40000008,
    STATUS_REGISTRY_RECOVERED = 0x40000009,
    STATUS_FT_READ_RECOVERY_FROM_BACKUP = 0x4000000a,
    STATUS_FT_WRITE_RECOVERY = 0x4000000b,
    STATUS_SERIAL_COUNTER_TIMEOUT = 0x4000000c,
    STATUS_NULL_LM_PASSWORD = 0x4000000d,
    STATUS_IMAGE_MACHINE_TYPE_MISMATCH = 0x4000000e,
    STATUS_RECEIVE_PARTIAL = 0x4000000f,
    STATUS_RECEIVE_EXPEDITED = 0x40000010,
    STATUS_RECEIVE_PARTIAL_EXPEDITED = 0x40000011,
    STATUS_EVENT_DONE = 0x40000012,
    STATUS_EVENT_PENDING = 0x40000013,
    STATUS_CHECKING_FILE_SYSTEM = 0x40000014,
    STATUS_FATAL_APP_EXIT = 0x40000015,
    STATUS_PREDEFINED_HANDLE = 0x40000016,
    STATUS_WAS_UNLOCKED = 0x40000017,
    STATUS_SERVICE_NOTIFICATION = 0x40000018,
    STATUS_WAS_LOCKED = 0x40000019,
    STATUS_LOG_HARD_ERROR = 0x4000001a,
    STATUS_ALREADY_WIN32 = 0x4000001b,
    STATUS_WX86_UNSIMULATE = 0x4000001c,
    STATUS_WX86_CONTINUE = 0x4000001d,
    STATUS_WX86_SINGLE_STEP = 0x4000001e,
    STATUS_WX86_BREAKPOINT = 0x4000001f,
    STATUS_WX86_EXCEPTION_CONTINUE = 0x40000020,
    STATUS_WX86_EXCEPTION_LASTCHANCE = 0x40000021,
    STATUS_WX86_EXCEPTION_CHAIN = 0x40000022,
    STATUS_IMAGE_MACHINE_TYPE_MISMATCH_EXE = 0x40000023,
    STATUS_NO_YIELD_PERFORMED = 0x40000024,
    STATUS_TIMER_RESUME_IGNORED = 0x40000025,
    STATUS_ARBITRATION_UNHANDLED = 0x40000026,
    STATUS_CARDBUS_NOT_SUPPORTED = 0x40000027,
    STATUS_WX86_CREATEWX86TIB = 0x40000028,
    STATUS_MP_PROCESSOR_MISMATCH = 0x40000029,
    STATUS_HIBERNATED = 0x4000002a,
    STATUS_RESUME_HIBERNATION = 0x4000002b,
    STATUS_FIRMWARE_UPDATED = 0x4000002c,
    STATUS_DRIVERS_LEAKING_LOCKED_PAGES = 0x4000002d,
    STATUS_MESSAGE_RETRIEVED = 0x4000002e,
    STATUS_SYSTEM_POWERSTATE_TRANSITION = 0x4000002f,
    STATUS_ALPC_CHECK_COMPLETION_LIST = 0x40000030,
    STATUS_SYSTEM_POWERSTATE_COMPLEX_TRANSITION = 0x40000031,
    STATUS_ACCESS_AUDIT_BY_POLICY = 0x40000032,
    STATUS_ABANDON_HIBERFILE = 0x40000033,
    STATUS_BIZRULES_NOT_ENABLED = 0x40000034,
    STATUS_FT_READ_FROM_COPY = 0x40000035,
    STATUS_IMAGE_AT_DIFFERENT_BASE = 0x40000036,
    STATUS_PATCH_DEFERRED = 0x40000037,
    STATUS_WAKE_SYSTEM = 0x40000294,
    STATUS_DS_SHUTTING_DOWN = 0x40000370,
    STATUS_DISK_REPAIR_REDIRECTED = 0x40000807,
    STATUS_SERVICES_FAILED_AUTOSTART = 0x4000a144,
    DBG_REPLY_LATER = 0x40010001,
    DBG_UNABLE_TO_PROVIDE_HANDLE = 0x40010002,
    DBG_TERMINATE_THREAD = 0x40010003,
    DBG_TERMINATE_PROCESS = 0x40010004,
    DBG_CONTROL_C = 0x40010005,
    DBG_PRINTEXCEPTION_C = 0x40010006,
    DBG_RIPEXCEPTION = 0x40010007,
    DBG_CONTROL_BREAK = 0x40010008,
    DBG_COMMAND_EXCEPTION = 0x40010009,
    DBG_PRINTEXCEPTION_WIDE_C = 0x4001000a,
    RPC_NT_UUID_LOCAL_ONLY = 0x40020056,
    RPC_NT_SEND_INCOMPLETE = 0x400200af,
    STATUS_CTX_CDM_CONNECT = 0x400a0004,
    STATUS_CTX_CDM_DISCONNECT = 0x400a0005,
    STATUS_SXS_RELEASE_ACTIVATION_CONTEXT = 0x4015000d,
    STATUS_HEURISTIC_DAMAGE_POSSIBLE = 0x40190001,
    STATUS_RECOVERY_NOT_NEEDED = 0x40190034,
    STATUS_RM_ALREADY_STARTED = 0x40190035,
    STATUS_LOG_NO_RESTART = 0x401a000c,
    STATUS_VIDEO_DRIVER_DEBUG_REPORT_REQUEST = 0x401b00ec,
    STATUS_GRAPHICS_PARTIAL_DATA_POPULATED = 0x401e000a,
    STATUS_GRAPHICS_SKIP_ALLOCATION_PREPARATION = 0x401e0201,
    STATUS_GRAPHICS_MODE_NOT_PINNED = 0x401e0307,
    STATUS_GRAPHICS_NO_PREFERRED_MODE = 0x401e031e,
    STATUS_GRAPHICS_DATASET_IS_EMPTY = 0x401e034b,
    STATUS_GRAPHICS_NO_MORE_ELEMENTS_IN_DATASET = 0x401e034c,
    STATUS_GRAPHICS_PATH_CONTENT_GEOMETRY_TRANSFORMATION_NOT_PINNED = 0x401e0351,
    STATUS_GRAPHICS_UNKNOWN_CHILD_STATUS = 0x401e042f,
    STATUS_GRAPHICS_LEADLINK_START_DEFERRED = 0x401e0437,
    STATUS_GRAPHICS_POLLING_TOO_FREQUENTLY = 0x401e0439,
    STATUS_GRAPHICS_START_DEFERRED = 0x401e043a,
    STATUS_GRAPHICS_DEPENDABLE_CHILD_STATUS = 0x401e043c,
    STATUS_NDIS_INDICATION_REQUIRED = 0x40230001,
    STATUS_PCP_UNSUPPORTED_PSS_SALT = 0x40292023,
    STATUS_GUARD_PAGE_VIOLATION = 0x80000001,
    STATUS_DATATYPE_MISALIGNMENT = 0x80000002,
    STATUS_BREAKPOINT = 0x80000003,
    STATUS_SINGLE_STEP = 0x80000004,
    STATUS_BUFFER_OVERFLOW = 0x80000005,
    STATUS_NO_MORE_FILES = 0x80000006,
    STATUS_WAKE_SYSTEM_DEBUGGER = 0x80000007,
    STATUS_HANDLES_CLOSED = 0x8000000a,
    STATUS_NO_INHERITANCE = 0x8000000b,
    STATUS_GUID_SUBSTITUTION_MADE = 0x8000000c,
    STATUS_PARTIAL_COPY = 0x8000000d,
    STATUS_DEVICE_PAPER_EMPTY = 0x8000000e,
    STATUS_DEVICE_POWERED_OFF = 0x8000000f,
    STATUS_DEVICE_OFF_LINE = 0x80000010,
    STATUS_DEVICE_BUSY = 0x80000011,
    STATUS_NO_MORE_EAS = 0x80000012,
    STATUS_INVALID_EA_NAME = 0x80000013,
    STATUS_EA_LIST_INCONSISTENT = 0x80000014,
    STATUS_INVALID_EA_FLAG = 0x80000015,
    STATUS_VERIFY_REQUIRED = 0x80000016,
    STATUS_EXTRANEOUS_INFORMATION = 0x80000017,
    STATUS_RXACT_COMMIT_NECESSARY = 0x80000018,
    STATUS_NO_MORE_ENTRIES = 0x8000001a,
    STATUS_FILEMARK_DETECTED = 0x8000001b,
    STATUS_MEDIA_CHANGED = 0x8000001c,
    STATUS_BUS_RESET = 0x8000001d,
    STATUS_END_OF_MEDIA = 0x8000001e,
    STATUS_BEGINNING_OF_MEDIA = 0x8000001f,
    STATUS_MEDIA_CHECK = 0x80000020,
    STATUS_SETMARK_DETECTED = 0x80000021,
    STATUS_NO_DATA_DETECTED = 0x80000022,
    STATUS_REDIRECTOR_HAS_OPEN_HANDLES = 0x80000023,
    STATUS_SERVER_HAS_OPEN_HANDLES = 0x80000024,
    STATUS_ALREADY_DISCONNECTED = 0x80000025,
    STATUS_LONGJUMP = 0x80000026,
    STATUS_CLEANER_CARTRIDGE_INSTALLED = 0x80000027,
    STATUS_PLUGPLAY_QUERY_VETOED = 0x80000028,
    STATUS_UNWIND_CONSOLIDATE = 0x80000029,
    STATUS_REGISTRY_HIVE_RECOVERED = 0x8000002a,
    STATUS_DLL_MIGHT_BE_INSECURE = 0x8000002b,
    STATUS_DLL_MIGHT_BE_INCOMPATIBLE = 0x8000002c,
    STATUS_STOPPED_ON_SYMLINK = 0x8000002d,
    STATUS_CANNOT_GRANT_REQUESTED_OPLOCK = 0x8000002e,
    STATUS_NO_ACE_CONDITION = 0x8000002f,
    STATUS_DEVICE_SUPPORT_IN_PROGRESS = 0x80000030,
    STATUS_DEVICE_POWER_CYCLE_REQUIRED = 0x80000031,
    STATUS_NO_WORK_DONE = 0x80000032,
    STATUS_RETURN_ADDRESS_HIJACK_ATTEMPT = 0x80000033,
    STATUS_DEVICE_REQUIRES_CLEANING = 0x80000288,
    STATUS_DEVICE_DOOR_OPEN = 0x80000289,
    STATUS_DATA_LOST_REPAIR = 0x80000803,
    STATUS_GPIO_INTERRUPT_ALREADY_UNMASKED = 0x8000a127,
    STATUS_CLOUD_FILE_PROPERTY_BLOB_CHECKSUM_MISMATCH = 0x8000cf00,
    STATUS_CLOUD_FILE_PROPERTY_BLOB_TOO_LARGE = 0x8000cf04,
    STATUS_CLOUD_FILE_TOO_MANY_PROPERTY_BLOBS = 0x8000cf05,
    DBG_EXCEPTION_NOT_HANDLED = 0x80010001,
    STATUS_CLUSTER_NODE_ALREADY_UP = 0x80130001,
    STATUS_CLUSTER_NODE_ALREADY_DOWN = 0x80130002,
    STATUS_CLUSTER_NETWORK_ALREADY_ONLINE = 0x80130003,
    STATUS_CLUSTER_NETWORK_ALREADY_OFFLINE = 0x80130004,
    STATUS_CLUSTER_NODE_ALREADY_MEMBER = 0x80130005,
    STATUS_COULD_NOT_RESIZE_LOG = 0x80190009,
    STATUS_NO_TXF_METADATA = 0x80190029,
    STATUS_CANT_RECOVER_WITH_HANDLE_OPEN = 0x80190031,
    STATUS_TXF_METADATA_ALREADY_PRESENT = 0x80190041,
    STATUS_TRANSACTION_SCOPE_CALLBACKS_NOT_SET = 0x80190042,
    STATUS_VIDEO_HUNG_DISPLAY_DRIVER_THREAD_RECOVERED = 0x801b00eb,
    STATUS_FLT_BUFFER_TOO_SMALL = 0x801c0001,
    STATUS_FVE_PARTIAL_METADATA = 0x80210001,
    STATUS_FVE_TRANSIENT_STATE = 0x80210002,
    STATUS_VID_REMOTE_NODE_PARENT_GPA_PAGES_USED = 0x80370001,
    STATUS_VOLMGR_INCOMPLETE_REGENERATION = 0x80380001,
    STATUS_VOLMGR_INCOMPLETE_DISK_MIGRATION = 0x80380002,
    STATUS_BCD_NOT_ALL_ENTRIES_IMPORTED = 0x80390001,
    STATUS_BCD_NOT_ALL_ENTRIES_SYNCHRONIZED = 0x80390003,
    STATUS_QUERY_STORAGE_ERROR = 0x803a0001,
    STATUS_GDI_HANDLE_LEAK = 0x803f0001,
    STATUS_SECUREBOOT_NOT_ENABLED = 0x80430006,
    STATUS_UNSUCCESSFUL = 0xc0000001,
    STATUS_NOT_IMPLEMENTED = 0xc0000002,
    STATUS_INVALID_INFO_CLASS = 0xc0000003,
    STATUS_INFO_LENGTH_MISMATCH = 0xc0000004,
    STATUS_ACCESS_VIOLATION = 0xc0000005,
    STATUS_IN_PAGE_ERROR = 0xc0000006,
    STATUS_PAGEFILE_QUOTA = 0xc0000007,
    STATUS_INVALID_HANDLE = 0xc0000008,
    STATUS_BAD_INITIAL_STACK = 0xc0000009,
    STATUS_BAD_INITIAL_PC = 0xc000000a,
    STATUS_INVALID_CID = 0xc000000b,
    STATUS_TIMER_NOT_CANCELED = 0xc000000c,
    STATUS_INVALID_PARAMETER = 0xc000000d,
    STATUS_NO_SUCH_DEVICE = 0xc000000e,
    STATUS_NO_SUCH_FILE = 0xc000000f,
    STATUS_INVALID_DEVICE_REQUEST = 0xc0000010,
    STATUS_END_OF_FILE = 0xc0000011,
    STATUS_WRONG_VOLUME = 0xc0000012,
    STATUS_NO_MEDIA_IN_DEVICE = 0xc0000013,
    STATUS_UNRECOGNIZED_MEDIA = 0xc0000014,
    STATUS_NONEXISTENT_SECTOR = 0xc0000015,
    STATUS_MORE_PROCESSING_REQUIRED = 0xc0000016,
    STATUS_NO_MEMORY = 0xc0000017,
    STATUS_CONFLICTING_ADDRESSES = 0xc0000018,
    STATUS_NOT_MAPPED_VIEW = 0xc0000019,
    STATUS_UNABLE_TO_FREE_VM = 0xc000001a,
    STATUS_UNABLE_TO_DELETE_SECTION = 0xc000001b,
    STATUS_INVALID_SYSTEM_SERVICE = 0xc000001c,
    STATUS_ILLEGAL_INSTRUCTION = 0xc000001d,
    STATUS_INVALID_LOCK_SEQUENCE = 0xc000001e,
    STATUS_INVALID_VIEW_SIZE = 0xc000001f,
    STATUS_INVALID_FILE_FOR_SECTION = 0xc0000020,
    STATUS_ALREADY_COMMITTED = 0xc0000021,
    STATUS_ACCESS_DENIED = 0xc0000022,
    STATUS_BUFFER_TOO_SMALL = 0xc0000023,
    STATUS_OBJECT_TYPE_MISMATCH = 0xc0000024,
    STATUS_NONCONTINUABLE_EXCEPTION = 0xc0000025,
    STATUS_INVALID_DISPOSITION = 0xc0000026,
    STATUS_UNWIND = 0xc0000027,
    STATUS_BAD_STACK = 0xc0000028,
    STATUS_INVALID_UNWIND_TARGET = 0xc0000029,
    STATUS_NOT_LOCKED = 0xc000002a,
    STATUS_PARITY_ERROR = 0xc000002b,
    STATUS_UNABLE_TO_DECOMMIT_VM = 0xc000002c,
    STATUS_NOT_COMMITTED = 0xc000002d,
    STATUS_INVALID_PORT_ATTRIBUTES = 0xc000002e,
    STATUS_PORT_MESSAGE_TOO_LONG = 0xc000002f,
    STATUS_INVALID_PARAMETER_MIX = 0xc0000030,
    STATUS_INVALID_QUOTA_LOWER = 0xc0000031,
    STATUS_DISK_CORRUPT_ERROR = 0xc0000032,
    STATUS_OBJECT_NAME_INVALID = 0xc0000033,
    STATUS_OBJECT_NAME_NOT_FOUND = 0xc0000034,
    STATUS_OBJECT_NAME_COLLISION = 0xc0000035,
    STATUS_PORT_DO_NOT_DISTURB = 0xc0000036,
    STATUS_PORT_DISCONNECTED = 0xc0000037,
    STATUS_DEVICE_ALREADY_ATTACHED = 0xc0000038,
    STATUS_OBJECT_PATH_INVALID = 0xc0000039,
    STATUS_OBJECT_PATH_NOT_FOUND = 0xc000003a,
    STATUS_OBJECT_PATH_SYNTAX_BAD = 0xc000003b,
    STATUS_DATA_OVERRUN = 0xc000003c,
    STATUS_DATA_LATE_ERROR = 0xc000003d,
    STATUS_DATA_ERROR = 0xc000003e,
    STATUS_CRC_ERROR = 0xc000003f,
    STATUS_SECTION_TOO_BIG = 0xc0000040,
    STATUS_PORT_CONNECTION_REFUSED = 0xc0000041,
    STATUS_INVALID_PORT_HANDLE = 0xc0000042,
    STATUS_SHARING_VIOLATION = 0xc0000043,
    STATUS_QUOTA_EXCEEDED = 0xc0000044,
    STATUS_INVALID_PAGE_PROTECTION = 0xc0000045,
    STATUS_MUTANT_NOT_OWNED = 0xc0000046,
    STATUS_SEMAPHORE_LIMIT_EXCEEDED = 0xc0000047,
    STATUS_PORT_ALREADY_SET = 0xc0000048,
    STATUS_SECTION_NOT_IMAGE = 0xc0000049,
    STATUS_SUSPEND_COUNT_EXCEEDED = 0xc000004a,
    STATUS_THREAD_IS_TERMINATING = 0xc000004b,
    STATUS_BAD_WORKING_SET_LIMIT = 0xc000004c,
    STATUS_INCOMPATIBLE_FILE_MAP = 0xc000004d,
    STATUS_SECTION_PROTECTION = 0xc000004e,
    STATUS_EAS_NOT_SUPPORTED = 0xc000004f,
    STATUS_EA_TOO_LARGE = 0xc0000050,
    STATUS_NONEXISTENT_EA_ENTRY = 0xc0000051,
    STATUS_NO_EAS_ON_FILE = 0xc0000052,
    STATUS_EA_CORRUPT_ERROR = 0xc0000053,
    STATUS_FILE_LOCK_CONFLICT = 0xc0000054,
    STATUS_LOCK_NOT_GRANTED = 0xc0000055,
    STATUS_DELETE_PENDING = 0xc0000056,
    STATUS_CTL_FILE_NOT_SUPPORTED = 0xc0000057,
    STATUS_UNKNOWN_REVISION = 0xc0000058,
    STATUS_REVISION_MISMATCH = 0xc0000059,
    STATUS_INVALID_OWNER = 0xc000005a,
    STATUS_INVALID_PRIMARY_GROUP = 0xc000005b,
    STATUS_NO_IMPERSONATION_TOKEN = 0xc000005c,
    STATUS_CANT_DISABLE_MANDATORY = 0xc000005d,
    STATUS_NO_LOGON_SERVERS = 0xc000005e,
    STATUS_NO_SUCH_LOGON_SESSION = 0xc000005f,
    STATUS_NO_SUCH_PRIVILEGE = 0xc0000060,
    STATUS_PRIVILEGE_NOT_HELD = 0xc0000061,
    STATUS_INVALID_ACCOUNT_NAME = 0xc0000062,
    STATUS_USER_EXISTS = 0xc0000063,
    STATUS_NO_SUCH_USER = 0xc0000064,
    STATUS_GROUP_EXISTS = 0xc0000065,
    STATUS_NO_SUCH_GROUP = 0xc0000066,
    STATUS_MEMBER_IN_GROUP = 0xc0000067,
    STATUS_MEMBER_NOT_IN_GROUP = 0xc0000068,
    STATUS_LAST_ADMIN = 0xc0000069,
    STATUS_WRONG_PASSWORD = 0xc000006a,
    STATUS_ILL_FORMED_PASSWORD = 0xc000006b,
    STATUS_PASSWORD_RESTRICTION = 0xc000006c,
    STATUS_LOGON_FAILURE = 0xc000006d,
    STATUS_ACCOUNT_RESTRICTION = 0xc000006e,
    STATUS_INVALID_LOGON_HOURS = 0xc000006f,
    STATUS_INVALID_WORKSTATION = 0xc0000070,
    STATUS_PASSWORD_EXPIRED = 0xc0000071,
    STATUS_ACCOUNT_DISABLED = 0xc0000072,
    STATUS_NONE_MAPPED = 0xc0000073,
    STATUS_TOO_MANY_LUIDS_REQUESTED = 0xc0000074,
    STATUS_LUIDS_EXHAUSTED = 0xc0000075,
    STATUS_INVALID_SUB_AUTHORITY = 0xc0000076,
    STATUS_INVALID_ACL = 0xc0000077,
    STATUS_INVALID_SID = 0xc0000078,
    STATUS_INVALID_SECURITY_DESCR = 0xc0000079,
    STATUS_PROCEDURE_NOT_FOUND = 0xc000007a,
    STATUS_INVALID_IMAGE_FORMAT = 0xc000007b,
    STATUS_NO_TOKEN = 0xc000007c,
    STATUS_BAD_INHERITANCE_ACL = 0xc000007d,
    STATUS_RANGE_NOT_LOCKED = 0xc000007e,
    STATUS_DISK_FULL = 0xc000007f,
    STATUS_SERVER_DISABLED = 0xc0000080,
    STATUS_SERVER_NOT_DISABLED = 0xc0000081,
    STATUS_TOO_MANY_GUIDS_REQUESTED = 0xc0000082,
    STATUS_GUIDS_EXHAUSTED = 0xc0000083,
    STATUS_INVALID_ID_AUTHORITY = 0xc0000084,
    STATUS_AGENTS_EXHAUSTED = 0xc0000085,
    STATUS_INVALID_VOLUME_LABEL = 0xc0000086,
    STATUS_SECTION_NOT_EXTENDED = 0xc0000087,
    STATUS_NOT_MAPPED_DATA = 0xc0000088,
    STATUS_RESOURCE_DATA_NOT_FOUND = 0xc0000089,
    STATUS_RESOURCE_TYPE_NOT_FOUND = 0xc000008a,
    STATUS_RESOURCE_NAME_NOT_FOUND = 0xc000008b,
    STATUS_ARRAY_BOUNDS_EXCEEDED = 0xc000008c,
    STATUS_FLOAT_DENORMAL_OPERAND = 0xc000008d,
    STATUS_FLOAT_DIVIDE_BY_ZERO = 0xc000008e,
    STATUS_FLOAT_INEXACT_RESULT = 0xc000008f,
    STATUS_FLOAT_INVALID_OPERATION = 0xc0000090,
    STATUS_FLOAT_OVERFLOW = 0xc0000091,
    STATUS_FLOAT_STACK_CHECK = 0xc0000092,
    STATUS_FLOAT_UNDERFLOW = 0xc0000093,
    STATUS_INTEGER_DIVIDE_BY_ZERO = 0xc0000094,
    STATUS_INTEGER_OVERFLOW = 0xc0000095,
    STATUS_PRIVILEGED_INSTRUCTION = 0xc0000096,
    STATUS_TOO_MANY_PAGING_FILES = 0xc0000097,
    STATUS_FILE_INVALID = 0xc0000098,
    STATUS_ALLOTTED_SPACE_EXCEEDED = 0xc0000099,
    STATUS_INSUFFICIENT_RESOURCES = 0xc000009a,
    STATUS_DFS_EXIT_PATH_FOUND = 0xc000009b,
    STATUS_DEVICE_DATA_ERROR = 0xc000009c,
    STATUS_DEVICE_NOT_CONNECTED = 0xc000009d,
    STATUS_DEVICE_POWER_FAILURE = 0xc000009e,
    STATUS_FREE_VM_NOT_AT_BASE = 0xc000009f,
    STATUS_MEMORY_NOT_ALLOCATED = 0xc00000a0,
    STATUS_WORKING_SET_QUOTA = 0xc00000a1,
    STATUS_MEDIA_WRITE_PROTECTED = 0xc00000a2,
    STATUS_DEVICE_NOT_READY = 0xc00000a3,
    STATUS_INVALID_GROUP_ATTRIBUTES = 0xc00000a4,
    STATUS_BAD_IMPERSONATION_LEVEL = 0xc00000a5,
    STATUS_CANT_OPEN_ANONYMOUS = 0xc00000a6,
    STATUS_BAD_VALIDATION_CLASS = 0xc00000a7,
    STATUS_BAD_TOKEN_TYPE = 0xc00000a8,
    STATUS_BAD_MASTER_BOOT_RECORD = 0xc00000a9,
    STATUS_INSTRUCTION_MISALIGNMENT = 0xc00000aa,
    STATUS_INSTANCE_NOT_AVAILABLE = 0xc00000ab,
    STATUS_PIPE_NOT_AVAILABLE = 0xc00000ac,
    STATUS_INVALID_PIPE_STATE = 0xc00000ad,
    STATUS_PIPE_BUSY = 0xc00000ae,
    STATUS_ILLEGAL_FUNCTION = 0xc00000af,
    STATUS_PIPE_DISCONNECTED = 0xc00000b0,
    STATUS_PIPE_CLOSING = 0xc00000b1,
    STATUS_PIPE_CONNECTED = 0xc00000b2,
    STATUS_PIPE_LISTENING = 0xc00000b3,
    STATUS_INVALID_READ_MODE = 0xc00000b4,
    STATUS_IO_TIMEOUT = 0xc00000b5,
    STATUS_FILE_FORCED_CLOSED = 0xc00000b6,
    STATUS_PROFILING_NOT_STARTED = 0xc00000b7,
    STATUS_PROFILING_NOT_STOPPED = 0xc00000b8,
    STATUS_COULD_NOT_INTERPRET = 0xc00000b9,
    STATUS_FILE_IS_A_DIRECTORY = 0xc00000ba,
    STATUS_NOT_SUPPORTED = 0xc00000bb,
    STATUS_REMOTE_NOT_LISTENING = 0xc00000bc,
    STATUS_DUPLICATE_NAME = 0xc00000bd,
    STATUS_BAD_NETWORK_PATH = 0xc00000be,
    STATUS_NETWORK_BUSY = 0xc00000bf,
    STATUS_DEVICE_DOES_NOT_EXIST = 0xc00000c0,
    STATUS_TOO_MANY_COMMANDS = 0xc00000c1,
    STATUS_ADAPTER_HARDWARE_ERROR = 0xc00000c2,
    STATUS_INVALID_NETWORK_RESPONSE = 0xc00000c3,
    STATUS_UNEXPECTED_NETWORK_ERROR = 0xc00000c4,
    STATUS_BAD_REMOTE_ADAPTER = 0xc00000c5,
    STATUS_PRINT_QUEUE_FULL = 0xc00000c6,
    STATUS_NO_SPOOL_SPACE = 0xc00000c7,
    STATUS_PRINT_CANCELLED = 0xc00000c8,
    STATUS_NETWORK_NAME_DELETED = 0xc00000c9,
    STATUS_NETWORK_ACCESS_DENIED = 0xc00000ca,
    STATUS_BAD_DEVICE_TYPE = 0xc00000cb,
    STATUS_BAD_NETWORK_NAME = 0xc00000cc,
    STATUS_TOO_MANY_NAMES = 0xc00000cd,
    STATUS_TOO_MANY_SESSIONS = 0xc00000ce,
    STATUS_SHARING_PAUSED = 0xc00000cf,
    STATUS_REQUEST_NOT_ACCEPTED = 0xc00000d0,
    STATUS_REDIRECTOR_PAUSED = 0xc00000d1,
    STATUS_NET_WRITE_FAULT = 0xc00000d2,
    STATUS_PROFILING_AT_LIMIT = 0xc00000d3,
    STATUS_NOT_SAME_DEVICE = 0xc00000d4,
    STATUS_FILE_RENAMED = 0xc00000d5,
    STATUS_VIRTUAL_CIRCUIT_CLOSED = 0xc00000d6,
    STATUS_NO_SECURITY_ON_OBJECT = 0xc00000d7,
    STATUS_CANT_WAIT = 0xc00000d8,
    STATUS_PIPE_EMPTY = 0xc00000d9,
    STATUS_CANT_ACCESS_DOMAIN_INFO = 0xc00000da,
    STATUS_CANT_TERMINATE_SELF = 0xc00000db,
    STATUS_INVALID_SERVER_STATE = 0xc00000dc,
    STATUS_INVALID_DOMAIN_STATE = 0xc00000dd,
    STATUS_INVALID_DOMAIN_ROLE = 0xc00000de,
    STATUS_NO_SUCH_DOMAIN = 0xc00000df,
    STATUS_DOMAIN_EXISTS = 0xc00000e0,
    STATUS_DOMAIN_LIMIT_EXCEEDED = 0xc00000e1,
    STATUS_OPLOCK_NOT_GRANTED = 0xc00000e2,
    STATUS_INVALID_OPLOCK_PROTOCOL = 0xc00000e3,
    STATUS_INTERNAL_DB_CORRUPTION = 0xc00000e4,
    STATUS_INTERNAL_ERROR = 0xc00000e5,
    STATUS_GENERIC_NOT_MAPPED = 0xc00000e6,
    STATUS_BAD_DESCRIPTOR_FORMAT = 0xc00000e7,
    STATUS_INVALID_USER_BUFFER = 0xc00000e8,
    STATUS_UNEXPECTED_IO_ERROR = 0xc00000e9,
    STATUS_UNEXPECTED_MM_CREATE_ERR = 0xc00000ea,
    STATUS_UNEXPECTED_MM_MAP_ERROR = 0xc00000eb,
    STATUS_UNEXPECTED_MM_EXTEND_ERR = 0xc00000ec,
    STATUS_NOT_LOGON_PROCESS = 0xc00000ed,
    STATUS_LOGON_SESSION_EXISTS = 0xc00000ee,
    STATUS_INVALID_PARAMETER_1 = 0xc00000ef,
    STATUS_INVALID_PARAMETER_2 = 0xc00000f0,
    STATUS_INVALID_PARAMETER_3 = 0xc00000f1,
    STATUS_INVALID_PARAMETER_4 = 0xc00000f2,
    STATUS_INVALID_PARAMETER_5 = 0xc00000f3,
    STATUS_INVALID_PARAMETER_6 = 0xc00000f4,
    STATUS_INVALID_PARAMETER_7 = 0xc00000f5,
    STATUS_INVALID_PARAMETER_8 = 0xc00000f6,
    STATUS_INVALID_PARAMETER_9 = 0xc00000f7,
    STATUS_INVALID_PARAMETER_10 = 0xc00000f8,
    STATUS_INVALID_PARAMETER_11 = 0xc00000f9,
    STATUS_INVALID_PARAMETER_12 = 0xc00000fa,
    STATUS_REDIRECTOR_NOT_STARTED = 0xc00000fb,
    STATUS_REDIRECTOR_STARTED = 0xc00000fc,
    STATUS_STACK_OVERFLOW = 0xc00000fd,
    STATUS_NO_SUCH_PACKAGE = 0xc00000fe,
    STATUS_BAD_FUNCTION_TABLE = 0xc00000ff,
    STATUS_VARIABLE_NOT_FOUND = 0xc0000100,
    STATUS_DIRECTORY_NOT_EMPTY = 0xc0000101,
    STATUS_FILE_CORRUPT_ERROR = 0xc0000102,
    STATUS_NOT_A_DIRECTORY = 0xc0000103,
    STATUS_BAD_LOGON_SESSION_STATE = 0xc0000104,
    STATUS_LOGON_SESSION_COLLISION = 0xc0000105,
    STATUS_NAME_TOO_LONG = 0xc0000106,
    STATUS_FILES_OPEN = 0xc0000107,
    STATUS_CONNECTION_IN_USE = 0xc0000108,
    STATUS_MESSAGE_NOT_FOUND = 0xc0000109,
    STATUS_PROCESS_IS_TERMINATING = 0xc000010a,
    STATUS_INVALID_LOGON_TYPE = 0xc000010b,
    STATUS_NO_GUID_TRANSLATION = 0xc000010c,
    STATUS_CANNOT_IMPERSONATE = 0xc000010d,
    STATUS_IMAGE_ALREADY_LOADED = 0xc000010e,
    STATUS_ABIOS_NOT_PRESENT = 0xc000010f,
    STATUS_ABIOS_LID_NOT_EXIST = 0xc0000110,
    STATUS_ABIOS_LID_ALREADY_OWNED = 0xc0000111,
    STATUS_ABIOS_NOT_LID_OWNER = 0xc0000112,
    STATUS_ABIOS_INVALID_COMMAND = 0xc0000113,
    STATUS_ABIOS_INVALID_LID = 0xc0000114,
    STATUS_ABIOS_SELECTOR_NOT_AVAILABLE = 0xc0000115,
    STATUS_ABIOS_INVALID_SELECTOR = 0xc0000116,
    STATUS_NO_LDT = 0xc0000117,
    STATUS_INVALID_LDT_SIZE = 0xc0000118,
    STATUS_INVALID_LDT_OFFSET = 0xc0000119,
    STATUS_INVALID_LDT_DESCRIPTOR = 0xc000011a,
    STATUS_INVALID_IMAGE_NE_FORMAT = 0xc000011b,
    STATUS_RXACT_INVALID_STATE = 0xc000011c,
    STATUS_RXACT_COMMIT_FAILURE = 0xc000011d,
    STATUS_MAPPED_FILE_SIZE_ZERO = 0xc000011e,
    STATUS_TOO_MANY_OPENED_FILES = 0xc000011f,
    STATUS_CANCELLED = 0xc0000120,
    STATUS_CANNOT_DELETE = 0xc0000121,
    STATUS_INVALID_COMPUTER_NAME = 0xc0000122,
    STATUS_FILE_DELETED = 0xc0000123,
    STATUS_SPECIAL_ACCOUNT = 0xc0000124,
    STATUS_SPECIAL_GROUP = 0xc0000125,
    STATUS_SPECIAL_USER = 0xc0000126,
    STATUS_MEMBERS_PRIMARY_GROUP = 0xc0000127,
    STATUS_FILE_CLOSED = 0xc0000128,
    STATUS_TOO_MANY_THREADS = 0xc0000129,
    STATUS_THREAD_NOT_IN_PROCESS = 0xc000012a,
    STATUS_TOKEN_ALREADY_IN_USE = 0xc000012b,
    STATUS_PAGEFILE_QUOTA_EXCEEDED = 0xc000012c,
    STATUS_COMMITMENT_LIMIT = 0xc000012d,
    STATUS_INVALID_IMAGE_LE_FORMAT = 0xc000012e,
    STATUS_INVALID_IMAGE_NOT_MZ = 0xc000012f,
    STATUS_INVALID_IMAGE_PROTECT = 0xc0000130,
    STATUS_INVALID_IMAGE_WIN_16 = 0xc0000131,
    STATUS_LOGON_SERVER_CONFLICT = 0xc0000132,
    STATUS_TIME_DIFFERENCE_AT_DC = 0xc0000133,
    STATUS_SYNCHRONIZATION_REQUIRED = 0xc0000134,
    STATUS_DLL_NOT_FOUND = 0xc0000135,
    STATUS_OPEN_FAILED = 0xc0000136,
    STATUS_IO_PRIVILEGE_FAILED = 0xc0000137,
    STATUS_ORDINAL_NOT_FOUND = 0xc0000138,
    STATUS_ENTRYPOINT_NOT_FOUND = 0xc0000139,
    STATUS_CONTROL_C_EXIT = 0xc000013a,
    STATUS_LOCAL_DISCONNECT = 0xc000013b,
    STATUS_REMOTE_DISCONNECT = 0xc000013c,
    STATUS_REMOTE_RESOURCES = 0xc000013d,
    STATUS_LINK_FAILED = 0xc000013e,
    STATUS_LINK_TIMEOUT = 0xc000013f,
    STATUS_INVALID_CONNECTION = 0xc0000140,
    STATUS_INVALID_ADDRESS = 0xc0000141,
    STATUS_DLL_INIT_FAILED = 0xc0000142,
    STATUS_MISSING_SYSTEMFILE = 0xc0000143,
    STATUS_UNHANDLED_EXCEPTION = 0xc0000144,
    STATUS_APP_INIT_FAILURE = 0xc0000145,
    STATUS_PAGEFILE_CREATE_FAILED = 0xc0000146,
    STATUS_NO_PAGEFILE = 0xc0000147,
    STATUS_INVALID_LEVEL = 0xc0000148,
    STATUS_WRONG_PASSWORD_CORE = 0xc0000149,
    STATUS_ILLEGAL_FLOAT_CONTEXT = 0xc000014a,
    STATUS_PIPE_BROKEN = 0xc000014b,
    STATUS_REGISTRY_CORRUPT = 0xc000014c,
    STATUS_REGISTRY_IO_FAILED = 0xc000014d,
    STATUS_NO_EVENT_PAIR = 0xc000014e,
    STATUS_UNRECOGNIZED_VOLUME = 0xc000014f,
    STATUS_SERIAL_NO_DEVICE_INITED = 0xc0000150,
    STATUS_NO_SUCH_ALIAS = 0xc0000151,
    STATUS_MEMBER_NOT_IN_ALIAS = 0xc0000152,
    STATUS_MEMBER_IN_ALIAS = 0xc0000153,
    STATUS_ALIAS_EXISTS = 0xc0000154,
    STATUS_LOGON_NOT_GRANTED = 0xc0000155,
    STATUS_TOO_MANY_SECRETS = 0xc0000156,
    STATUS_SECRET_TOO_LONG = 0xc0000157,
    STATUS_INTERNAL_DB_ERROR = 0xc0000158,
    STATUS_FULLSCREEN_MODE = 0xc0000159,
    STATUS_TOO_MANY_CONTEXT_IDS = 0xc000015a,
    STATUS_LOGON_TYPE_NOT_GRANTED = 0xc000015b,
    STATUS_NOT_REGISTRY_FILE = 0xc000015c,
    STATUS_NT_CROSS_ENCRYPTION_REQUIRED = 0xc000015d,
    STATUS_DOMAIN_CTRLR_CONFIG_ERROR = 0xc000015e,
    STATUS_FT_MISSING_MEMBER = 0xc000015f,
    STATUS_ILL_FORMED_SERVICE_ENTRY = 0xc0000160,
    STATUS_ILLEGAL_CHARACTER = 0xc0000161,
    STATUS_UNMAPPABLE_CHARACTER = 0xc0000162,
    STATUS_UNDEFINED_CHARACTER = 0xc0000163,
    STATUS_FLOPPY_VOLUME = 0xc0000164,
    STATUS_FLOPPY_ID_MARK_NOT_FOUND = 0xc0000165,
    STATUS_FLOPPY_WRONG_CYLINDER = 0xc0000166,
    STATUS_FLOPPY_UNKNOWN_ERROR = 0xc0000167,
    STATUS_FLOPPY_BAD_REGISTERS = 0xc0000168,
    STATUS_DISK_RECALIBRATE_FAILED = 0xc0000169,
    STATUS_DISK_OPERATION_FAILED = 0xc000016a,
    STATUS_DISK_RESET_FAILED = 0xc000016b,
    STATUS_SHARED_IRQ_BUSY = 0xc000016c,
    STATUS_FT_ORPHANING = 0xc000016d,
    STATUS_BIOS_FAILED_TO_CONNECT_INTERRUPT = 0xc000016e,
    STATUS_PARTITION_FAILURE = 0xc0000172,
    STATUS_INVALID_BLOCK_LENGTH = 0xc0000173,
    STATUS_DEVICE_NOT_PARTITIONED = 0xc0000174,
    STATUS_UNABLE_TO_LOCK_MEDIA = 0xc0000175,
    STATUS_UNABLE_TO_UNLOAD_MEDIA = 0xc0000176,
    STATUS_EOM_OVERFLOW = 0xc0000177,
    STATUS_NO_MEDIA = 0xc0000178,
    STATUS_NO_SUCH_MEMBER = 0xc000017a,
    STATUS_INVALID_MEMBER = 0xc000017b,
    STATUS_KEY_DELETED = 0xc000017c,
    STATUS_NO_LOG_SPACE = 0xc000017d,
    STATUS_TOO_MANY_SIDS = 0xc000017e,
    STATUS_LM_CROSS_ENCRYPTION_REQUIRED = 0xc000017f,
    STATUS_KEY_HAS_CHILDREN = 0xc0000180,
    STATUS_CHILD_MUST_BE_VOLATILE = 0xc0000181,
    STATUS_DEVICE_CONFIGURATION_ERROR = 0xc0000182,
    STATUS_DRIVER_INTERNAL_ERROR = 0xc0000183,
    STATUS_INVALID_DEVICE_STATE = 0xc0000184,
    STATUS_IO_DEVICE_ERROR = 0xc0000185,
    STATUS_DEVICE_PROTOCOL_ERROR = 0xc0000186,
    STATUS_BACKUP_CONTROLLER = 0xc0000187,
    STATUS_LOG_FILE_FULL = 0xc0000188,
    STATUS_TOO_LATE = 0xc0000189,
    STATUS_NO_TRUST_LSA_SECRET = 0xc000018a,
    STATUS_NO_TRUST_SAM_ACCOUNT = 0xc000018b,
    STATUS_TRUSTED_DOMAIN_FAILURE = 0xc000018c,
    STATUS_TRUSTED_RELATIONSHIP_FAILURE = 0xc000018d,
    STATUS_EVENTLOG_FILE_CORRUPT = 0xc000018e,
    STATUS_EVENTLOG_CANT_START = 0xc000018f,
    STATUS_TRUST_FAILURE = 0xc0000190,
    STATUS_MUTANT_LIMIT_EXCEEDED = 0xc0000191,
    STATUS_NETLOGON_NOT_STARTED = 0xc0000192,
    STATUS_ACCOUNT_EXPIRED = 0xc0000193,
    STATUS_POSSIBLE_DEADLOCK = 0xc0000194,
    STATUS_NETWORK_CREDENTIAL_CONFLICT = 0xc0000195,
    STATUS_REMOTE_SESSION_LIMIT = 0xc0000196,
    STATUS_EVENTLOG_FILE_CHANGED = 0xc0000197,
    STATUS_NOLOGON_INTERDOMAIN_TRUST_ACCOUNT = 0xc0000198,
    STATUS_NOLOGON_WORKSTATION_TRUST_ACCOUNT = 0xc0000199,
    STATUS_NOLOGON_SERVER_TRUST_ACCOUNT = 0xc000019a,
    STATUS_DOMAIN_TRUST_INCONSISTENT = 0xc000019b,
    STATUS_FS_DRIVER_REQUIRED = 0xc000019c,
    STATUS_IMAGE_ALREADY_LOADED_AS_DLL = 0xc000019d,
    STATUS_INCOMPATIBLE_WITH_GLOBAL_SHORT_NAME_REGISTRY_SETTING = 0xc000019e,
    STATUS_SHORT_NAMES_NOT_ENABLED_ON_VOLUME = 0xc000019f,
    STATUS_SECURITY_STREAM_IS_INCONSISTENT = 0xc00001a0,
    STATUS_INVALID_LOCK_RANGE = 0xc00001a1,
    STATUS_INVALID_ACE_CONDITION = 0xc00001a2,
    STATUS_IMAGE_SUBSYSTEM_NOT_PRESENT = 0xc00001a3,
    STATUS_NOTIFICATION_GUID_ALREADY_DEFINED = 0xc00001a4,
    STATUS_INVALID_EXCEPTION_HANDLER = 0xc00001a5,
    STATUS_DUPLICATE_PRIVILEGES = 0xc00001a6,
    STATUS_NOT_ALLOWED_ON_SYSTEM_FILE = 0xc00001a7,
    STATUS_REPAIR_NEEDED = 0xc00001a8,
    STATUS_QUOTA_NOT_ENABLED = 0xc00001a9,
    STATUS_NO_APPLICATION_PACKAGE = 0xc00001aa,
    STATUS_FILE_METADATA_OPTIMIZATION_IN_PROGRESS = 0xc00001ab,
    STATUS_NOT_SAME_OBJECT = 0xc00001ac,
    STATUS_FATAL_MEMORY_EXHAUSTION = 0xc00001ad,
    STATUS_ERROR_PROCESS_NOT_IN_JOB = 0xc00001ae,
    STATUS_CPU_SET_INVALID = 0xc00001af,
    STATUS_IO_DEVICE_INVALID_DATA = 0xc00001b0,
    STATUS_IO_UNALIGNED_WRITE = 0xc00001b1,
    STATUS_CONTROL_STACK_VIOLATION = 0xc00001b2,
    STATUS_NETWORK_OPEN_RESTRICTION = 0xc0000201,
    STATUS_NO_USER_SESSION_KEY = 0xc0000202,
    STATUS_USER_SESSION_DELETED = 0xc0000203,
    STATUS_RESOURCE_LANG_NOT_FOUND = 0xc0000204,
    STATUS_INSUFF_SERVER_RESOURCES = 0xc0000205,
    STATUS_INVALID_BUFFER_SIZE = 0xc0000206,
    STATUS_INVALID_ADDRESS_COMPONENT = 0xc0000207,
    STATUS_INVALID_ADDRESS_WILDCARD = 0xc0000208,
    STATUS_TOO_MANY_ADDRESSES = 0xc0000209,
    STATUS_ADDRESS_ALREADY_EXISTS = 0xc000020a,
    STATUS_ADDRESS_CLOSED = 0xc000020b,
    STATUS_CONNECTION_DISCONNECTED = 0xc000020c,
    STATUS_CONNECTION_RESET = 0xc000020d,
    STATUS_TOO_MANY_NODES = 0xc000020e,
    STATUS_TRANSACTION_ABORTED = 0xc000020f,
    STATUS_TRANSACTION_TIMED_OUT = 0xc0000210,
    STATUS_TRANSACTION_NO_RELEASE = 0xc0000211,
    STATUS_TRANSACTION_NO_MATCH = 0xc0000212,
    STATUS_TRANSACTION_RESPONDED = 0xc0000213,
    STATUS_TRANSACTION_INVALID_ID = 0xc0000214,
    STATUS_TRANSACTION_INVALID_TYPE = 0xc0000215,
    STATUS_NOT_SERVER_SESSION = 0xc0000216,
    STATUS_NOT_CLIENT_SESSION = 0xc0000217,
    STATUS_CANNOT_LOAD_REGISTRY_FILE = 0xc0000218,
    STATUS_DEBUG_ATTACH_FAILED = 0xc0000219,
    STATUS_SYSTEM_PROCESS_TERMINATED = 0xc000021a,
    STATUS_DATA_NOT_ACCEPTED = 0xc000021b,
    STATUS_NO_BROWSER_SERVERS_FOUND = 0xc000021c,
    STATUS_VDM_HARD_ERROR = 0xc000021d,
    STATUS_DRIVER_CANCEL_TIMEOUT = 0xc000021e,
    STATUS_REPLY_MESSAGE_MISMATCH = 0xc000021f,
    STATUS_MAPPED_ALIGNMENT = 0xc0000220,
    STATUS_IMAGE_CHECKSUM_MISMATCH = 0xc0000221,
    STATUS_LOST_WRITEBEHIND_DATA = 0xc0000222,
    STATUS_CLIENT_SERVER_PARAMETERS_INVALID = 0xc0000223,
    STATUS_PASSWORD_MUST_CHANGE = 0xc0000224,
    STATUS_NOT_FOUND = 0xc0000225,
    STATUS_NOT_TINY_STREAM = 0xc0000226,
    STATUS_RECOVERY_FAILURE = 0xc0000227,
    STATUS_STACK_OVERFLOW_READ = 0xc0000228,
    STATUS_FAIL_CHECK = 0xc0000229,
    STATUS_DUPLICATE_OBJECTID = 0xc000022a,
    STATUS_OBJECTID_EXISTS = 0xc000022b,
    STATUS_CONVERT_TO_LARGE = 0xc000022c,
    STATUS_RETRY = 0xc000022d,
    STATUS_FOUND_OUT_OF_SCOPE = 0xc000022e,
    STATUS_ALLOCATE_BUCKET = 0xc000022f,
    STATUS_PROPSET_NOT_FOUND = 0xc0000230,
    STATUS_MARSHALL_OVERFLOW = 0xc0000231,
    STATUS_INVALID_VARIANT = 0xc0000232,
    STATUS_DOMAIN_CONTROLLER_NOT_FOUND = 0xc0000233,
    STATUS_ACCOUNT_LOCKED_OUT = 0xc0000234,
    STATUS_HANDLE_NOT_CLOSABLE = 0xc0000235,
    STATUS_CONNECTION_REFUSED = 0xc0000236,
    STATUS_GRACEFUL_DISCONNECT = 0xc0000237,
    STATUS_ADDRESS_ALREADY_ASSOCIATED = 0xc0000238,
    STATUS_ADDRESS_NOT_ASSOCIATED = 0xc0000239,
    STATUS_CONNECTION_INVALID = 0xc000023a,
    STATUS_CONNECTION_ACTIVE = 0xc000023b,
    STATUS_NETWORK_UNREACHABLE = 0xc000023c,
    STATUS_HOST_UNREACHABLE = 0xc000023d,
    STATUS_PROTOCOL_UNREACHABLE = 0xc000023e,
    STATUS_PORT_UNREACHABLE = 0xc000023f,
    STATUS_REQUEST_ABORTED = 0xc0000240,
    STATUS_CONNECTION_ABORTED = 0xc0000241,
    STATUS_BAD_COMPRESSION_BUFFER = 0xc0000242,
    STATUS_USER_MAPPED_FILE = 0xc0000243,
    STATUS_AUDIT_FAILED = 0xc0000244,
    STATUS_TIMER_RESOLUTION_NOT_SET = 0xc0000245,
    STATUS_CONNECTION_COUNT_LIMIT = 0xc0000246,
    STATUS_LOGIN_TIME_RESTRICTION = 0xc0000247,
    STATUS_LOGIN_WKSTA_RESTRICTION = 0xc0000248,
    STATUS_IMAGE_MP_UP_MISMATCH = 0xc0000249,
    STATUS_INSUFFICIENT_LOGON_INFO = 0xc0000250,
    STATUS_BAD_DLL_ENTRYPOINT = 0xc0000251,
    STATUS_BAD_SERVICE_ENTRYPOINT = 0xc0000252,
    STATUS_LPC_REPLY_LOST = 0xc0000253,
    STATUS_IP_ADDRESS_CONFLICT1 = 0xc0000254,
    STATUS_IP_ADDRESS_CONFLICT2 = 0xc0000255,
    STATUS_REGISTRY_QUOTA_LIMIT = 0xc0000256,
    STATUS_PATH_NOT_COVERED = 0xc0000257,
    STATUS_NO_CALLBACK_ACTIVE = 0xc0000258,
    STATUS_LICENSE_QUOTA_EXCEEDED = 0xc0000259,
    STATUS_PWD_TOO_SHORT = 0xc000025a,
    STATUS_PWD_TOO_RECENT = 0xc000025b,
    STATUS_PWD_HISTORY_CONFLICT = 0xc000025c,
    STATUS_PLUGPLAY_NO_DEVICE = 0xc000025e,
    STATUS_UNSUPPORTED_COMPRESSION = 0xc000025f,
    STATUS_INVALID_HW_PROFILE = 0xc0000260,
    STATUS_INVALID_PLUGPLAY_DEVICE_PATH = 0xc0000261,
    STATUS_DRIVER_ORDINAL_NOT_FOUND = 0xc0000262,
    STATUS_DRIVER_ENTRYPOINT_NOT_FOUND = 0xc0000263,
    STATUS_RESOURCE_NOT_OWNED = 0xc0000264,
    STATUS_TOO_MANY_LINKS = 0xc0000265,
    STATUS_QUOTA_LIST_INCONSISTENT = 0xc0000266,
    STATUS_FILE_IS_OFFLINE = 0xc0000267,
    STATUS_EVALUATION_EXPIRATION = 0xc0000268,
    STATUS_ILLEGAL_DLL_RELOCATION = 0xc0000269,
    STATUS_LICENSE_VIOLATION = 0xc000026a,
    STATUS_DLL_INIT_FAILED_LOGOFF = 0xc000026b,
    STATUS_DRIVER_UNABLE_TO_LOAD = 0xc000026c,
    STATUS_DFS_UNAVAILABLE = 0xc000026d,
    STATUS_VOLUME_DISMOUNTED = 0xc000026e,
    STATUS_WX86_INTERNAL_ERROR = 0xc000026f,
    STATUS_WX86_FLOAT_STACK_CHECK = 0xc0000270,
    STATUS_VALIDATE_CONTINUE = 0xc0000271,
    STATUS_NO_MATCH = 0xc0000272,
    STATUS_NO_MORE_MATCHES = 0xc0000273,
    STATUS_NOT_A_REPARSE_POINT = 0xc0000275,
    STATUS_IO_REPARSE_TAG_INVALID = 0xc0000276,
    STATUS_IO_REPARSE_TAG_MISMATCH = 0xc0000277,
    STATUS_IO_REPARSE_DATA_INVALID = 0xc0000278,
    STATUS_IO_REPARSE_TAG_NOT_HANDLED = 0xc0000279,
    STATUS_PWD_TOO_LONG = 0xc000027a,
    STATUS_STOWED_EXCEPTION = 0xc000027b,
    STATUS_CONTEXT_STOWED_EXCEPTION = 0xc000027c,
    STATUS_REPARSE_POINT_NOT_RESOLVED = 0xc0000280,
    STATUS_DIRECTORY_IS_A_REPARSE_POINT = 0xc0000281,
    STATUS_RANGE_LIST_CONFLICT = 0xc0000282,
    STATUS_SOURCE_ELEMENT_EMPTY = 0xc0000283,
    STATUS_DESTINATION_ELEMENT_FULL = 0xc0000284,
    STATUS_ILLEGAL_ELEMENT_ADDRESS = 0xc0000285,
    STATUS_MAGAZINE_NOT_PRESENT = 0xc0000286,
    STATUS_REINITIALIZATION_NEEDED = 0xc0000287,
    STATUS_ENCRYPTION_FAILED = 0xc000028a,
    STATUS_DECRYPTION_FAILED = 0xc000028b,
    STATUS_RANGE_NOT_FOUND = 0xc000028c,
    STATUS_NO_RECOVERY_POLICY = 0xc000028d,
    STATUS_NO_EFS = 0xc000028e,
    STATUS_WRONG_EFS = 0xc000028f,
    STATUS_NO_USER_KEYS = 0xc0000290,
    STATUS_FILE_NOT_ENCRYPTED = 0xc0000291,
    STATUS_NOT_EXPORT_FORMAT = 0xc0000292,
    STATUS_FILE_ENCRYPTED = 0xc0000293,
    STATUS_WMI_GUID_NOT_FOUND = 0xc0000295,
    STATUS_WMI_INSTANCE_NOT_FOUND = 0xc0000296,
    STATUS_WMI_ITEMID_NOT_FOUND = 0xc0000297,
    STATUS_WMI_TRY_AGAIN = 0xc0000298,
    STATUS_SHARED_POLICY = 0xc0000299,
    STATUS_POLICY_OBJECT_NOT_FOUND = 0xc000029a,
    STATUS_POLICY_ONLY_IN_DS = 0xc000029b,
    STATUS_VOLUME_NOT_UPGRADED = 0xc000029c,
    STATUS_REMOTE_STORAGE_NOT_ACTIVE = 0xc000029d,
    STATUS_REMOTE_STORAGE_MEDIA_ERROR = 0xc000029e,
    STATUS_NO_TRACKING_SERVICE = 0xc000029f,
    STATUS_SERVER_SID_MISMATCH = 0xc00002a0,
    STATUS_DS_NO_ATTRIBUTE_OR_VALUE = 0xc00002a1,
    STATUS_DS_INVALID_ATTRIBUTE_SYNTAX = 0xc00002a2,
    STATUS_DS_ATTRIBUTE_TYPE_UNDEFINED = 0xc00002a3,
    STATUS_DS_ATTRIBUTE_OR_VALUE_EXISTS = 0xc00002a4,
    STATUS_DS_BUSY = 0xc00002a5,
    STATUS_DS_UNAVAILABLE = 0xc00002a6,
    STATUS_DS_NO_RIDS_ALLOCATED = 0xc00002a7,
    STATUS_DS_NO_MORE_RIDS = 0xc00002a8,
    STATUS_DS_INCORRECT_ROLE_OWNER = 0xc00002a9,
    STATUS_DS_RIDMGR_INIT_ERROR = 0xc00002aa,
    STATUS_DS_OBJ_CLASS_VIOLATION = 0xc00002ab,
    STATUS_DS_CANT_ON_NON_LEAF = 0xc00002ac,
    STATUS_DS_CANT_ON_RDN = 0xc00002ad,
    STATUS_DS_CANT_MOD_OBJ_CLASS = 0xc00002ae,
    STATUS_DS_CROSS_DOM_MOVE_FAILED = 0xc00002af,
    STATUS_DS_GC_NOT_AVAILABLE = 0xc00002b0,
    STATUS_DIRECTORY_SERVICE_REQUIRED = 0xc00002b1,
    STATUS_REPARSE_ATTRIBUTE_CONFLICT = 0xc00002b2,
    STATUS_CANT_ENABLE_DENY_ONLY = 0xc00002b3,
    STATUS_FLOAT_MULTIPLE_FAULTS = 0xc00002b4,
    STATUS_FLOAT_MULTIPLE_TRAPS = 0xc00002b5,
    STATUS_DEVICE_REMOVED = 0xc00002b6,
    STATUS_JOURNAL_DELETE_IN_PROGRESS = 0xc00002b7,
    STATUS_JOURNAL_NOT_ACTIVE = 0xc00002b8,
    STATUS_NOINTERFACE = 0xc00002b9,
    STATUS_DS_RIDMGR_DISABLED = 0xc00002ba,
    STATUS_DS_ADMIN_LIMIT_EXCEEDED = 0xc00002c1,
    STATUS_DRIVER_FAILED_SLEEP = 0xc00002c2,
    STATUS_MUTUAL_AUTHENTICATION_FAILED = 0xc00002c3,
    STATUS_CORRUPT_SYSTEM_FILE = 0xc00002c4,
    STATUS_DATATYPE_MISALIGNMENT_ERROR = 0xc00002c5,
    STATUS_WMI_READ_ONLY = 0xc00002c6,
    STATUS_WMI_SET_FAILURE = 0xc00002c7,
    STATUS_COMMITMENT_MINIMUM = 0xc00002c8,
    STATUS_REG_NAT_CONSUMPTION = 0xc00002c9,
    STATUS_TRANSPORT_FULL = 0xc00002ca,
    STATUS_DS_SAM_INIT_FAILURE = 0xc00002cb,
    STATUS_ONLY_IF_CONNECTED = 0xc00002cc,
    STATUS_DS_SENSITIVE_GROUP_VIOLATION = 0xc00002cd,
    STATUS_PNP_RESTART_ENUMERATION = 0xc00002ce,
    STATUS_JOURNAL_ENTRY_DELETED = 0xc00002cf,
    STATUS_DS_CANT_MOD_PRIMARYGROUPID = 0xc00002d0,
    STATUS_SYSTEM_IMAGE_BAD_SIGNATURE = 0xc00002d1,
    STATUS_PNP_REBOOT_REQUIRED = 0xc00002d2,
    STATUS_POWER_STATE_INVALID = 0xc00002d3,
    STATUS_DS_INVALID_GROUP_TYPE = 0xc00002d4,
    STATUS_DS_NO_NEST_GLOBALGROUP_IN_MIXEDDOMAIN = 0xc00002d5,
    STATUS_DS_NO_NEST_LOCALGROUP_IN_MIXEDDOMAIN = 0xc00002d6,
    STATUS_DS_GLOBAL_CANT_HAVE_LOCAL_MEMBER = 0xc00002d7,
    STATUS_DS_GLOBAL_CANT_HAVE_UNIVERSAL_MEMBER = 0xc00002d8,
    STATUS_DS_UNIVERSAL_CANT_HAVE_LOCAL_MEMBER = 0xc00002d9,
    STATUS_DS_GLOBAL_CANT_HAVE_CROSSDOMAIN_MEMBER = 0xc00002da,
    STATUS_DS_LOCAL_CANT_HAVE_CROSSDOMAIN_LOCAL_MEMBER = 0xc00002db,
    STATUS_DS_HAVE_PRIMARY_MEMBERS = 0xc00002dc,
    STATUS_WMI_NOT_SUPPORTED = 0xc00002dd,
    STATUS_INSUFFICIENT_POWER = 0xc00002de,
    STATUS_SAM_NEED_BOOTKEY_PASSWORD = 0xc00002df,
    STATUS_SAM_NEED_BOOTKEY_FLOPPY = 0xc00002e0,
    STATUS_DS_CANT_START = 0xc00002e1,
    STATUS_DS_INIT_FAILURE = 0xc00002e2,
    STATUS_SAM_INIT_FAILURE = 0xc00002e3,
    STATUS_DS_GC_REQUIRED = 0xc00002e4,
    STATUS_DS_LOCAL_MEMBER_OF_LOCAL_ONLY = 0xc00002e5,
    STATUS_DS_NO_FPO_IN_UNIVERSAL_GROUPS = 0xc00002e6,
    STATUS_DS_MACHINE_ACCOUNT_QUOTA_EXCEEDED = 0xc00002e7,
    STATUS_MULTIPLE_FAULT_VIOLATION = 0xc00002e8,
    STATUS_CURRENT_DOMAIN_NOT_ALLOWED = 0xc00002e9,
    STATUS_CANNOT_MAKE = 0xc00002ea,
    STATUS_SYSTEM_SHUTDOWN = 0xc00002eb,
    STATUS_DS_INIT_FAILURE_CONSOLE = 0xc00002ec,
    STATUS_DS_SAM_INIT_FAILURE_CONSOLE = 0xc00002ed,
    STATUS_UNFINISHED_CONTEXT_DELETED = 0xc00002ee,
    STATUS_NO_TGT_REPLY = 0xc00002ef,
    STATUS_OBJECTID_NOT_FOUND = 0xc00002f0,
    STATUS_NO_IP_ADDRESSES = 0xc00002f1,
    STATUS_WRONG_CREDENTIAL_HANDLE = 0xc00002f2,
    STATUS_CRYPTO_SYSTEM_INVALID = 0xc00002f3,
    STATUS_MAX_REFERRALS_EXCEEDED = 0xc00002f4,
    STATUS_MUST_BE_KDC = 0xc00002f5,
    STATUS_STRONG_CRYPTO_NOT_SUPPORTED = 0xc00002f6,
    STATUS_TOO_MANY_PRINCIPALS = 0xc00002f7,
    STATUS_NO_PA_DATA = 0xc00002f8,
    STATUS_PKINIT_NAME_MISMATCH = 0xc00002f9,
    STATUS_SMARTCARD_LOGON_REQUIRED = 0xc00002fa,
    STATUS_KDC_INVALID_REQUEST = 0xc00002fb,
    STATUS_KDC_UNABLE_TO_REFER = 0xc00002fc,
    STATUS_KDC_UNKNOWN_ETYPE = 0xc00002fd,
    STATUS_SHUTDOWN_IN_PROGRESS = 0xc00002fe,
    STATUS_SERVER_SHUTDOWN_IN_PROGRESS = 0xc00002ff,
    STATUS_NOT_SUPPORTED_ON_SBS = 0xc0000300,
    STATUS_WMI_GUID_DISCONNECTED = 0xc0000301,
    STATUS_WMI_ALREADY_DISABLED = 0xc0000302,
    STATUS_WMI_ALREADY_ENABLED = 0xc0000303,
    STATUS_MFT_TOO_FRAGMENTED = 0xc0000304,
    STATUS_COPY_PROTECTION_FAILURE = 0xc0000305,
    STATUS_CSS_AUTHENTICATION_FAILURE = 0xc0000306,
    STATUS_CSS_KEY_NOT_PRESENT = 0xc0000307,
    STATUS_CSS_KEY_NOT_ESTABLISHED = 0xc0000308,
    STATUS_CSS_SCRAMBLED_SECTOR = 0xc0000309,
    STATUS_CSS_REGION_MISMATCH = 0xc000030a,
    STATUS_CSS_RESETS_EXHAUSTED = 0xc000030b,
    STATUS_PASSWORD_CHANGE_REQUIRED = 0xc000030c,
    STATUS_LOST_MODE_LOGON_RESTRICTION = 0xc000030d,
    STATUS_PKINIT_FAILURE = 0xc0000320,
    STATUS_SMARTCARD_SUBSYSTEM_FAILURE = 0xc0000321,
    STATUS_NO_KERB_KEY = 0xc0000322,
    STATUS_HOST_DOWN = 0xc0000350,
    STATUS_UNSUPPORTED_PREAUTH = 0xc0000351,
    STATUS_EFS_ALG_BLOB_TOO_BIG = 0xc0000352,
    STATUS_PORT_NOT_SET = 0xc0000353,
    STATUS_DEBUGGER_INACTIVE = 0xc0000354,
    STATUS_DS_VERSION_CHECK_FAILURE = 0xc0000355,
    STATUS_AUDITING_DISABLED = 0xc0000356,
    STATUS_PRENT4_MACHINE_ACCOUNT = 0xc0000357,
    STATUS_DS_AG_CANT_HAVE_UNIVERSAL_MEMBER = 0xc0000358,
    STATUS_INVALID_IMAGE_WIN_32 = 0xc0000359,
    STATUS_INVALID_IMAGE_WIN_64 = 0xc000035a,
    STATUS_BAD_BINDINGS = 0xc000035b,
    STATUS_NETWORK_SESSION_EXPIRED = 0xc000035c,
    STATUS_APPHELP_BLOCK = 0xc000035d,
    STATUS_ALL_SIDS_FILTERED = 0xc000035e,
    STATUS_NOT_SAFE_MODE_DRIVER = 0xc000035f,
    STATUS_ACCESS_DISABLED_BY_POLICY_DEFAULT = 0xc0000361,
    STATUS_ACCESS_DISABLED_BY_POLICY_PATH = 0xc0000362,
    STATUS_ACCESS_DISABLED_BY_POLICY_PUBLISHER = 0xc0000363,
    STATUS_ACCESS_DISABLED_BY_POLICY_OTHER = 0xc0000364,
    STATUS_FAILED_DRIVER_ENTRY = 0xc0000365,
    STATUS_DEVICE_ENUMERATION_ERROR = 0xc0000366,
    STATUS_MOUNT_POINT_NOT_RESOLVED = 0xc0000368,
    STATUS_INVALID_DEVICE_OBJECT_PARAMETER = 0xc0000369,
    STATUS_MCA_OCCURED = 0xc000036a,
    STATUS_DRIVER_BLOCKED_CRITICAL = 0xc000036b,
    STATUS_DRIVER_BLOCKED = 0xc000036c,
    STATUS_DRIVER_DATABASE_ERROR = 0xc000036d,
    STATUS_SYSTEM_HIVE_TOO_LARGE = 0xc000036e,
    STATUS_INVALID_IMPORT_OF_NON_DLL = 0xc000036f,
    STATUS_NO_SECRETS = 0xc0000371,
    STATUS_ACCESS_DISABLED_NO_SAFER_UI_BY_POLICY = 0xc0000372,
    STATUS_FAILED_STACK_SWITCH = 0xc0000373,
    STATUS_HEAP_CORRUPTION = 0xc0000374,
    STATUS_SMARTCARD_WRONG_PIN = 0xc0000380,
    STATUS_SMARTCARD_CARD_BLOCKED = 0xc0000381,
    STATUS_SMARTCARD_CARD_NOT_AUTHENTICATED = 0xc0000382,
    STATUS_SMARTCARD_NO_CARD = 0xc0000383,
    STATUS_SMARTCARD_NO_KEY_CONTAINER = 0xc0000384,
    STATUS_SMARTCARD_NO_CERTIFICATE = 0xc0000385,
    STATUS_SMARTCARD_NO_KEYSET = 0xc0000386,
    STATUS_SMARTCARD_IO_ERROR = 0xc0000387,
    STATUS_DOWNGRADE_DETECTED = 0xc0000388,
    STATUS_SMARTCARD_CERT_REVOKED = 0xc0000389,
    STATUS_ISSUING_CA_UNTRUSTED = 0xc000038a,
    STATUS_REVOCATION_OFFLINE_C = 0xc000038b,
    STATUS_PKINIT_CLIENT_FAILURE = 0xc000038c,
    STATUS_SMARTCARD_CERT_EXPIRED = 0xc000038d,
    STATUS_DRIVER_FAILED_PRIOR_UNLOAD = 0xc000038e,
    STATUS_SMARTCARD_SILENT_CONTEXT = 0xc000038f,
    STATUS_PER_USER_TRUST_QUOTA_EXCEEDED = 0xc0000401,
    STATUS_ALL_USER_TRUST_QUOTA_EXCEEDED = 0xc0000402,
    STATUS_USER_DELETE_TRUST_QUOTA_EXCEEDED = 0xc0000403,
    STATUS_DS_NAME_NOT_UNIQUE = 0xc0000404,
    STATUS_DS_DUPLICATE_ID_FOUND = 0xc0000405,
    STATUS_DS_GROUP_CONVERSION_ERROR = 0xc0000406,
    STATUS_VOLSNAP_PREPARE_HIBERNATE = 0xc0000407,
    STATUS_USER2USER_REQUIRED = 0xc0000408,
    STATUS_STACK_BUFFER_OVERRUN = 0xc0000409,
    STATUS_NO_S4U_PROT_SUPPORT = 0xc000040a,
    STATUS_CROSSREALM_DELEGATION_FAILURE = 0xc000040b,
    STATUS_REVOCATION_OFFLINE_KDC = 0xc000040c,
    STATUS_ISSUING_CA_UNTRUSTED_KDC = 0xc000040d,
    STATUS_KDC_CERT_EXPIRED = 0xc000040e,
    STATUS_KDC_CERT_REVOKED = 0xc000040f,
    STATUS_PARAMETER_QUOTA_EXCEEDED = 0xc0000410,
    STATUS_HIBERNATION_FAILURE = 0xc0000411,
    STATUS_DELAY_LOAD_FAILED = 0xc0000412,
    STATUS_AUTHENTICATION_FIREWALL_FAILED = 0xc0000413,
    STATUS_VDM_DISALLOWED = 0xc0000414,
    STATUS_HUNG_DISPLAY_DRIVER_THREAD = 0xc0000415,
    STATUS_INSUFFICIENT_RESOURCE_FOR_SPECIFIED_SHARED_SECTION_SIZE = 0xc0000416,
    STATUS_INVALID_CRUNTIME_PARAMETER = 0xc0000417,
    STATUS_NTLM_BLOCKED = 0xc0000418,
    STATUS_DS_SRC_SID_EXISTS_IN_FOREST = 0xc0000419,
    STATUS_DS_DOMAIN_NAME_EXISTS_IN_FOREST = 0xc000041a,
    STATUS_DS_FLAT_NAME_EXISTS_IN_FOREST = 0xc000041b,
    STATUS_INVALID_USER_PRINCIPAL_NAME = 0xc000041c,
    STATUS_FATAL_USER_CALLBACK_EXCEPTION = 0xc000041d,
    STATUS_ASSERTION_FAILURE = 0xc0000420,
    STATUS_VERIFIER_STOP = 0xc0000421,
    STATUS_CALLBACK_POP_STACK = 0xc0000423,
    STATUS_INCOMPATIBLE_DRIVER_BLOCKED = 0xc0000424,
    STATUS_HIVE_UNLOADED = 0xc0000425,
    STATUS_COMPRESSION_DISABLED = 0xc0000426,
    STATUS_FILE_SYSTEM_LIMITATION = 0xc0000427,
    STATUS_INVALID_IMAGE_HASH = 0xc0000428,
    STATUS_NOT_CAPABLE = 0xc0000429,
    STATUS_REQUEST_OUT_OF_SEQUENCE = 0xc000042a,
    STATUS_IMPLEMENTATION_LIMIT = 0xc000042b,
    STATUS_ELEVATION_REQUIRED = 0xc000042c,
    STATUS_NO_SECURITY_CONTEXT = 0xc000042d,
    STATUS_PKU2U_CERT_FAILURE = 0xc000042f,
    STATUS_BEYOND_VDL = 0xc0000432,
    STATUS_ENCOUNTERED_WRITE_IN_PROGRESS = 0xc0000433,
    STATUS_PTE_CHANGED = 0xc0000434,
    STATUS_PURGE_FAILED = 0xc0000435,
    STATUS_CRED_REQUIRES_CONFIRMATION = 0xc0000440,
    STATUS_CS_ENCRYPTION_INVALID_SERVER_RESPONSE = 0xc0000441,
    STATUS_CS_ENCRYPTION_UNSUPPORTED_SERVER = 0xc0000442,
    STATUS_CS_ENCRYPTION_EXISTING_ENCRYPTED_FILE = 0xc0000443,
    STATUS_CS_ENCRYPTION_NEW_ENCRYPTED_FILE = 0xc0000444,
    STATUS_CS_ENCRYPTION_FILE_NOT_CSE = 0xc0000445,
    STATUS_INVALID_LABEL = 0xc0000446,
    STATUS_DRIVER_PROCESS_TERMINATED = 0xc0000450,
    STATUS_AMBIGUOUS_SYSTEM_DEVICE = 0xc0000451,
    STATUS_SYSTEM_DEVICE_NOT_FOUND = 0xc0000452,
    STATUS_RESTART_BOOT_APPLICATION = 0xc0000453,
    STATUS_INSUFFICIENT_NVRAM_RESOURCES = 0xc0000454,
    STATUS_INVALID_SESSION = 0xc0000455,
    STATUS_THREAD_ALREADY_IN_SESSION = 0xc0000456,
    STATUS_THREAD_NOT_IN_SESSION = 0xc0000457,
    STATUS_INVALID_WEIGHT = 0xc0000458,
    STATUS_REQUEST_PAUSED = 0xc0000459,
    STATUS_NO_RANGES_PROCESSED = 0xc0000460,
    STATUS_DISK_RESOURCES_EXHAUSTED = 0xc0000461,
    STATUS_NEEDS_REMEDIATION = 0xc0000462,
    STATUS_DEVICE_FEATURE_NOT_SUPPORTED = 0xc0000463,
    STATUS_DEVICE_UNREACHABLE = 0xc0000464,
    STATUS_INVALID_TOKEN = 0xc0000465,
    STATUS_SERVER_UNAVAILABLE = 0xc0000466,
    STATUS_FILE_NOT_AVAILABLE = 0xc0000467,
    STATUS_DEVICE_INSUFFICIENT_RESOURCES = 0xc0000468,
    STATUS_PACKAGE_UPDATING = 0xc0000469,
    STATUS_NOT_READ_FROM_COPY = 0xc000046a,
    STATUS_FT_WRITE_FAILURE = 0xc000046b,
    STATUS_FT_DI_SCAN_REQUIRED = 0xc000046c,
    STATUS_OBJECT_NOT_EXTERNALLY_BACKED = 0xc000046d,
    STATUS_EXTERNAL_BACKING_PROVIDER_UNKNOWN = 0xc000046e,
    STATUS_COMPRESSION_NOT_BENEFICIAL = 0xc000046f,
    STATUS_DATA_CHECKSUM_ERROR = 0xc0000470,
    STATUS_INTERMIXED_KERNEL_EA_OPERATION = 0xc0000471,
    STATUS_TRIM_READ_ZERO_NOT_SUPPORTED = 0xc0000472,
    STATUS_TOO_MANY_SEGMENT_DESCRIPTORS = 0xc0000473,
    STATUS_INVALID_OFFSET_ALIGNMENT = 0xc0000474,
    STATUS_INVALID_FIELD_IN_PARAMETER_LIST = 0xc0000475,
    STATUS_OPERATION_IN_PROGRESS = 0xc0000476,
    STATUS_INVALID_INITIATOR_TARGET_PATH = 0xc0000477,
    STATUS_SCRUB_DATA_DISABLED = 0xc0000478,
    STATUS_NOT_REDUNDANT_STORAGE = 0xc0000479,
    STATUS_RESIDENT_FILE_NOT_SUPPORTED = 0xc000047a,
    STATUS_COMPRESSED_FILE_NOT_SUPPORTED = 0xc000047b,
    STATUS_DIRECTORY_NOT_SUPPORTED = 0xc000047c,
    STATUS_IO_OPERATION_TIMEOUT = 0xc000047d,
    STATUS_SYSTEM_NEEDS_REMEDIATION = 0xc000047e,
    STATUS_APPX_INTEGRITY_FAILURE_CLR_NGEN = 0xc000047f,
    STATUS_SHARE_UNAVAILABLE = 0xc0000480,
    STATUS_APISET_NOT_HOSTED = 0xc0000481,
    STATUS_APISET_NOT_PRESENT = 0xc0000482,
    STATUS_DEVICE_HARDWARE_ERROR = 0xc0000483,
    STATUS_FIRMWARE_SLOT_INVALID = 0xc0000484,
    STATUS_FIRMWARE_IMAGE_INVALID = 0xc0000485,
    STATUS_STORAGE_TOPOLOGY_ID_MISMATCH = 0xc0000486,
    STATUS_WIM_NOT_BOOTABLE = 0xc0000487,
    STATUS_BLOCKED_BY_PARENTAL_CONTROLS = 0xc0000488,
    STATUS_NEEDS_REGISTRATION = 0xc0000489,
    STATUS_QUOTA_ACTIVITY = 0xc000048a,
    STATUS_CALLBACK_INVOKE_INLINE = 0xc000048b,
    STATUS_BLOCK_TOO_MANY_REFERENCES = 0xc000048c,
    STATUS_MARKED_TO_DISALLOW_WRITES = 0xc000048d,
    STATUS_NETWORK_ACCESS_DENIED_EDP = 0xc000048e,
    STATUS_ENCLAVE_FAILURE = 0xc000048f,
    STATUS_PNP_NO_COMPAT_DRIVERS = 0xc0000490,
    STATUS_PNP_DRIVER_PACKAGE_NOT_FOUND = 0xc0000491,
    STATUS_PNP_DRIVER_CONFIGURATION_NOT_FOUND = 0xc0000492,
    STATUS_PNP_DRIVER_CONFIGURATION_INCOMPLETE = 0xc0000493,
    STATUS_PNP_FUNCTION_DRIVER_REQUIRED = 0xc0000494,
    STATUS_PNP_DEVICE_CONFIGURATION_PENDING = 0xc0000495,
    STATUS_DEVICE_HINT_NAME_BUFFER_TOO_SMALL = 0xc0000496,
    STATUS_PACKAGE_NOT_AVAILABLE = 0xc0000497,
    STATUS_DEVICE_IN_MAINTENANCE = 0xc0000499,
    STATUS_NOT_SUPPORTED_ON_DAX = 0xc000049a,
    STATUS_FREE_SPACE_TOO_FRAGMENTED = 0xc000049b,
    STATUS_DAX_MAPPING_EXISTS = 0xc000049c,
    STATUS_CHILD_PROCESS_BLOCKED = 0xc000049d,
    STATUS_STORAGE_LOST_DATA_PERSISTENCE = 0xc000049e,
    STATUS_VRF_CFG_AND_IO_ENABLED = 0xc000049f,
    STATUS_PARTITION_TERMINATING = 0xc00004a0,
    STATUS_EXTERNAL_SYSKEY_NOT_SUPPORTED = 0xc00004a1,
    STATUS_ENCLAVE_VIOLATION = 0xc00004a2,
    STATUS_FILE_PROTECTED_UNDER_DPL = 0xc00004a3,
    STATUS_VOLUME_NOT_CLUSTER_ALIGNED = 0xc00004a4,
    STATUS_NO_PHYSICALLY_ALIGNED_FREE_SPACE_FOUND = 0xc00004a5,
    STATUS_APPX_FILE_NOT_ENCRYPTED = 0xc00004a6,
    STATUS_RWRAW_ENCRYPTED_FILE_NOT_ENCRYPTED = 0xc00004a7,
    STATUS_RWRAW_ENCRYPTED_INVALID_EDATAINFO_FILEOFFSET = 0xc00004a8,
    STATUS_RWRAW_ENCRYPTED_INVALID_EDATAINFO_FILERANGE = 0xc00004a9,
    STATUS_RWRAW_ENCRYPTED_INVALID_EDATAINFO_PARAMETER = 0xc00004aa,
    STATUS_FT_READ_FAILURE = 0xc00004ab,
    STATUS_PATCH_CONFLICT = 0xc00004ac,
    STATUS_STORAGE_RESERVE_ID_INVALID = 0xc00004ad,
    STATUS_STORAGE_RESERVE_DOES_NOT_EXIST = 0xc00004ae,
    STATUS_STORAGE_RESERVE_ALREADY_EXISTS = 0xc00004af,
    STATUS_STORAGE_RESERVE_NOT_EMPTY = 0xc00004b0,
    STATUS_NOT_A_DAX_VOLUME = 0xc00004b1,
    STATUS_NOT_DAX_MAPPABLE = 0xc00004b2,
    STATUS_CASE_DIFFERING_NAMES_IN_DIR = 0xc00004b3,
    STATUS_FILE_NOT_SUPPORTED = 0xc00004b4,
    STATUS_NOT_SUPPORTED_WITH_BTT = 0xc00004b5,
    STATUS_ENCRYPTION_DISABLED = 0xc00004b6,
    STATUS_ENCRYPTING_METADATA_DISALLOWED = 0xc00004b7,
    STATUS_CANT_CLEAR_ENCRYPTION_FLAG = 0xc00004b8,
    STATUS_UNSATISFIED_DEPENDENCIES = 0xc00004b9,
    STATUS_CASE_SENSITIVE_PATH = 0xc00004ba,
    STATUS_HAS_SYSTEM_CRITICAL_FILES = 0xc00004bd,
    STATUS_INVALID_TASK_NAME = 0xc0000500,
    STATUS_INVALID_TASK_INDEX = 0xc0000501,
    STATUS_THREAD_ALREADY_IN_TASK = 0xc0000502,
    STATUS_CALLBACK_BYPASS = 0xc0000503,
    STATUS_UNDEFINED_SCOPE = 0xc0000504,
    STATUS_INVALID_CAP = 0xc0000505,
    STATUS_NOT_GUI_PROCESS = 0xc0000506,
    STATUS_DEVICE_HUNG = 0xc0000507,
    STATUS_CONTAINER_ASSIGNED = 0xc0000508,
    STATUS_JOB_NO_CONTAINER = 0xc0000509,
    STATUS_DEVICE_UNRESPONSIVE = 0xc000050a,
    STATUS_REPARSE_POINT_ENCOUNTERED = 0xc000050b,
    STATUS_ATTRIBUTE_NOT_PRESENT = 0xc000050c,
    STATUS_NOT_A_TIERED_VOLUME = 0xc000050d,
    STATUS_ALREADY_HAS_STREAM_ID = 0xc000050e,
    STATUS_JOB_NOT_EMPTY = 0xc000050f,
    STATUS_ALREADY_INITIALIZED = 0xc0000510,
    STATUS_ENCLAVE_NOT_TERMINATED = 0xc0000511,
    STATUS_ENCLAVE_IS_TERMINATING = 0xc0000512,
    STATUS_SMB1_NOT_AVAILABLE = 0xc0000513,
    STATUS_SMR_GARBAGE_COLLECTION_REQUIRED = 0xc0000514,
    STATUS_INTERRUPTED = 0xc0000515,
    STATUS_THREAD_NOT_RUNNING = 0xc0000516,
    STATUS_FAIL_FAST_EXCEPTION = 0xc0000602,
    STATUS_IMAGE_CERT_REVOKED = 0xc0000603,
    STATUS_DYNAMIC_CODE_BLOCKED = 0xc0000604,
    STATUS_IMAGE_CERT_EXPIRED = 0xc0000605,
    STATUS_STRICT_CFG_VIOLATION = 0xc0000606,
    STATUS_SET_CONTEXT_DENIED = 0xc000060a,
    STATUS_CROSS_PARTITION_VIOLATION = 0xc000060b,
    STATUS_PORT_CLOSED = 0xc0000700,
    STATUS_MESSAGE_LOST = 0xc0000701,
    STATUS_INVALID_MESSAGE = 0xc0000702,
    STATUS_REQUEST_CANCELED = 0xc0000703,
    STATUS_RECURSIVE_DISPATCH = 0xc0000704,
    STATUS_LPC_RECEIVE_BUFFER_EXPECTED = 0xc0000705,
    STATUS_LPC_INVALID_CONNECTION_USAGE = 0xc0000706,
    STATUS_LPC_REQUESTS_NOT_ALLOWED = 0xc0000707,
    STATUS_RESOURCE_IN_USE = 0xc0000708,
    STATUS_HARDWARE_MEMORY_ERROR = 0xc0000709,
    STATUS_THREADPOOL_HANDLE_EXCEPTION = 0xc000070a,
    STATUS_THREADPOOL_SET_EVENT_ON_COMPLETION_FAILED = 0xc000070b,
    STATUS_THREADPOOL_RELEASE_SEMAPHORE_ON_COMPLETION_FAILED = 0xc000070c,
    STATUS_THREADPOOL_RELEASE_MUTEX_ON_COMPLETION_FAILED = 0xc000070d,
    STATUS_THREADPOOL_FREE_LIBRARY_ON_COMPLETION_FAILED = 0xc000070e,
    STATUS_THREADPOOL_RELEASED_DURING_OPERATION = 0xc000070f,
    STATUS_CALLBACK_RETURNED_WHILE_IMPERSONATING = 0xc0000710,
    STATUS_APC_RETURNED_WHILE_IMPERSONATING = 0xc0000711,
    STATUS_PROCESS_IS_PROTECTED = 0xc0000712,
    STATUS_MCA_EXCEPTION = 0xc0000713,
    STATUS_CERTIFICATE_MAPPING_NOT_UNIQUE = 0xc0000714,
    STATUS_SYMLINK_CLASS_DISABLED = 0xc0000715,
    STATUS_INVALID_IDN_NORMALIZATION = 0xc0000716,
    STATUS_NO_UNICODE_TRANSLATION = 0xc0000717,
    STATUS_ALREADY_REGISTERED = 0xc0000718,
    STATUS_CONTEXT_MISMATCH = 0xc0000719,
    STATUS_PORT_ALREADY_HAS_COMPLETION_LIST = 0xc000071a,
    STATUS_CALLBACK_RETURNED_THREAD_PRIORITY = 0xc000071b,
    STATUS_INVALID_THREAD = 0xc000071c,
    STATUS_CALLBACK_RETURNED_TRANSACTION = 0xc000071d,
    STATUS_CALLBACK_RETURNED_LDR_LOCK = 0xc000071e,
    STATUS_CALLBACK_RETURNED_LANG = 0xc000071f,
    STATUS_CALLBACK_RETURNED_PRI_BACK = 0xc0000720,
    STATUS_CALLBACK_RETURNED_THREAD_AFFINITY = 0xc0000721,
    STATUS_LPC_HANDLE_COUNT_EXCEEDED = 0xc0000722,
    STATUS_EXECUTABLE_MEMORY_WRITE = 0xc0000723,
    STATUS_KERNEL_EXECUTABLE_MEMORY_WRITE = 0xc0000724,
    STATUS_ATTACHED_EXECUTABLE_MEMORY_WRITE = 0xc0000725,
    STATUS_TRIGGERED_EXECUTABLE_MEMORY_WRITE = 0xc0000726,
    STATUS_DISK_REPAIR_DISABLED = 0xc0000800,
    STATUS_DS_DOMAIN_RENAME_IN_PROGRESS = 0xc0000801,
    STATUS_DISK_QUOTA_EXCEEDED = 0xc0000802,
    STATUS_CONTENT_BLOCKED = 0xc0000804,
    STATUS_BAD_CLUSTERS = 0xc0000805,
    STATUS_VOLUME_DIRTY = 0xc0000806,
    STATUS_DISK_REPAIR_UNSUCCESSFUL = 0xc0000808,
    STATUS_CORRUPT_LOG_OVERFULL = 0xc0000809,
    STATUS_CORRUPT_LOG_CORRUPTED = 0xc000080a,
    STATUS_CORRUPT_LOG_UNAVAILABLE = 0xc000080b,
    STATUS_CORRUPT_LOG_DELETED_FULL = 0xc000080c,
    STATUS_CORRUPT_LOG_CLEARED = 0xc000080d,
    STATUS_ORPHAN_NAME_EXHAUSTED = 0xc000080e,
    STATUS_PROACTIVE_SCAN_IN_PROGRESS = 0xc000080f,
    STATUS_ENCRYPTED_IO_NOT_POSSIBLE = 0xc0000810,
    STATUS_CORRUPT_LOG_UPLEVEL_RECORDS = 0xc0000811,
    STATUS_FILE_CHECKED_OUT = 0xc0000901,
    STATUS_CHECKOUT_REQUIRED = 0xc0000902,
    STATUS_BAD_FILE_TYPE = 0xc0000903,
    STATUS_FILE_TOO_LARGE = 0xc0000904,
    STATUS_FORMS_AUTH_REQUIRED = 0xc0000905,
    STATUS_VIRUS_INFECTED = 0xc0000906,
    STATUS_VIRUS_DELETED = 0xc0000907,
    STATUS_BAD_MCFG_TABLE = 0xc0000908,
    STATUS_CANNOT_BREAK_OPLOCK = 0xc0000909,
    STATUS_BAD_KEY = 0xc000090a,
    STATUS_BAD_DATA = 0xc000090b,
    STATUS_NO_KEY = 0xc000090c,
    STATUS_FILE_HANDLE_REVOKED = 0xc0000910,
    STATUS_WOW_ASSERTION = 0xc0009898,
    STATUS_INVALID_SIGNATURE = 0xc000a000,
    STATUS_HMAC_NOT_SUPPORTED = 0xc000a001,
    STATUS_AUTH_TAG_MISMATCH = 0xc000a002,
    STATUS_INVALID_STATE_TRANSITION = 0xc000a003,
    STATUS_INVALID_KERNEL_INFO_VERSION = 0xc000a004,
    STATUS_INVALID_PEP_INFO_VERSION = 0xc000a005,
    STATUS_HANDLE_REVOKED = 0xc000a006,
    STATUS_EOF_ON_GHOSTED_RANGE = 0xc000a007,
    STATUS_CC_NEEDS_CALLBACK_SECTION_DRAIN = 0xc000a008,
    STATUS_IPSEC_QUEUE_OVERFLOW = 0xc000a010,
    STATUS_ND_QUEUE_OVERFLOW = 0xc000a011,
    STATUS_HOPLIMIT_EXCEEDED = 0xc000a012,
    STATUS_PROTOCOL_NOT_SUPPORTED = 0xc000a013,
    STATUS_FASTPATH_REJECTED = 0xc000a014,
    STATUS_LOST_WRITEBEHIND_DATA_NETWORK_DISCONNECTED = 0xc000a080,
    STATUS_LOST_WRITEBEHIND_DATA_NETWORK_SERVER_ERROR = 0xc000a081,
    STATUS_LOST_WRITEBEHIND_DATA_LOCAL_DISK_ERROR = 0xc000a082,
    STATUS_XML_PARSE_ERROR = 0xc000a083,
    STATUS_XMLDSIG_ERROR = 0xc000a084,
    STATUS_WRONG_COMPARTMENT = 0xc000a085,
    STATUS_AUTHIP_FAILURE = 0xc000a086,
    STATUS_DS_OID_MAPPED_GROUP_CANT_HAVE_MEMBERS = 0xc000a087,
    STATUS_DS_OID_NOT_FOUND = 0xc000a088,
    STATUS_INCORRECT_ACCOUNT_TYPE = 0xc000a089,
    STATUS_HASH_NOT_SUPPORTED = 0xc000a100,
    STATUS_HASH_NOT_PRESENT = 0xc000a101,
    STATUS_SECONDARY_IC_PROVIDER_NOT_REGISTERED = 0xc000a121,
    STATUS_GPIO_CLIENT_INFORMATION_INVALID = 0xc000a122,
    STATUS_GPIO_VERSION_NOT_SUPPORTED = 0xc000a123,
    STATUS_GPIO_INVALID_REGISTRATION_PACKET = 0xc000a124,
    STATUS_GPIO_OPERATION_DENIED = 0xc000a125,
    STATUS_GPIO_INCOMPATIBLE_CONNECT_MODE = 0xc000a126,
    STATUS_CANNOT_SWITCH_RUNLEVEL = 0xc000a141,
    STATUS_INVALID_RUNLEVEL_SETTING = 0xc000a142,
    STATUS_RUNLEVEL_SWITCH_TIMEOUT = 0xc000a143,
    STATUS_RUNLEVEL_SWITCH_AGENT_TIMEOUT = 0xc000a145,
    STATUS_RUNLEVEL_SWITCH_IN_PROGRESS = 0xc000a146,
    STATUS_NOT_APPCONTAINER = 0xc000a200,
    STATUS_NOT_SUPPORTED_IN_APPCONTAINER = 0xc000a201,
    STATUS_INVALID_PACKAGE_SID_LENGTH = 0xc000a202,
    STATUS_LPAC_ACCESS_DENIED = 0xc000a203,
    STATUS_ADMINLESS_ACCESS_DENIED = 0xc000a204,
    STATUS_APP_DATA_NOT_FOUND = 0xc000a281,
    STATUS_APP_DATA_EXPIRED = 0xc000a282,
    STATUS_APP_DATA_CORRUPT = 0xc000a283,
    STATUS_APP_DATA_LIMIT_EXCEEDED = 0xc000a284,
    STATUS_APP_DATA_REBOOT_REQUIRED = 0xc000a285,
    STATUS_OFFLOAD_READ_FLT_NOT_SUPPORTED = 0xc000a2a1,
    STATUS_OFFLOAD_WRITE_FLT_NOT_SUPPORTED = 0xc000a2a2,
    STATUS_OFFLOAD_READ_FILE_NOT_SUPPORTED = 0xc000a2a3,
    STATUS_OFFLOAD_WRITE_FILE_NOT_SUPPORTED = 0xc000a2a4,
    STATUS_WOF_WIM_HEADER_CORRUPT = 0xc000a2a5,
    STATUS_WOF_WIM_RESOURCE_TABLE_CORRUPT = 0xc000a2a6,
    STATUS_WOF_FILE_RESOURCE_TABLE_CORRUPT = 0xc000a2a7,
    STATUS_CIMFS_IMAGE_CORRUPT = 0xc000c001,
    STATUS_FILE_SYSTEM_VIRTUALIZATION_UNAVAILABLE = 0xc000ce01,
    STATUS_FILE_SYSTEM_VIRTUALIZATION_METADATA_CORRUPT = 0xc000ce02,
    STATUS_FILE_SYSTEM_VIRTUALIZATION_BUSY = 0xc000ce03,
    STATUS_FILE_SYSTEM_VIRTUALIZATION_PROVIDER_UNKNOWN = 0xc000ce04,
    STATUS_FILE_SYSTEM_VIRTUALIZATION_INVALID_OPERATION = 0xc000ce05,
    STATUS_CLOUD_FILE_SYNC_ROOT_METADATA_CORRUPT = 0xc000cf00,
    STATUS_CLOUD_FILE_PROVIDER_NOT_RUNNING = 0xc000cf01,
    STATUS_CLOUD_FILE_METADATA_CORRUPT = 0xc000cf02,
    STATUS_CLOUD_FILE_METADATA_TOO_LARGE = 0xc000cf03,
    STATUS_CLOUD_FILE_PROPERTY_VERSION_NOT_SUPPORTED = 0xc000cf06,
    STATUS_NOT_A_CLOUD_FILE = 0xc000cf07,
    STATUS_CLOUD_FILE_NOT_IN_SYNC = 0xc000cf08,
    STATUS_CLOUD_FILE_ALREADY_CONNECTED = 0xc000cf09,
    STATUS_CLOUD_FILE_NOT_SUPPORTED = 0xc000cf0a,
    STATUS_CLOUD_FILE_INVALID_REQUEST = 0xc000cf0b,
    STATUS_CLOUD_FILE_READ_ONLY_VOLUME = 0xc000cf0c,
    STATUS_CLOUD_FILE_CONNECTED_PROVIDER_ONLY = 0xc000cf0d,
    STATUS_CLOUD_FILE_VALIDATION_FAILED = 0xc000cf0e,
    STATUS_CLOUD_FILE_AUTHENTICATION_FAILED = 0xc000cf0f,
    STATUS_CLOUD_FILE_INSUFFICIENT_RESOURCES = 0xc000cf10,
    STATUS_CLOUD_FILE_NETWORK_UNAVAILABLE = 0xc000cf11,
    STATUS_CLOUD_FILE_UNSUCCESSFUL = 0xc000cf12,
    STATUS_CLOUD_FILE_NOT_UNDER_SYNC_ROOT = 0xc000cf13,
    STATUS_CLOUD_FILE_IN_USE = 0xc000cf14,
    STATUS_CLOUD_FILE_PINNED = 0xc000cf15,
    STATUS_CLOUD_FILE_REQUEST_ABORTED = 0xc000cf16,
    STATUS_CLOUD_FILE_PROPERTY_CORRUPT = 0xc000cf17,
    STATUS_CLOUD_FILE_ACCESS_DENIED = 0xc000cf18,
    STATUS_CLOUD_FILE_INCOMPATIBLE_HARDLINKS = 0xc000cf19,
    STATUS_CLOUD_FILE_PROPERTY_LOCK_CONFLICT = 0xc000cf1a,
    STATUS_CLOUD_FILE_REQUEST_CANCELED = 0xc000cf1b,
    STATUS_CLOUD_FILE_PROVIDER_TERMINATED = 0xc000cf1d,
    STATUS_NOT_A_CLOUD_SYNC_ROOT = 0xc000cf1e,
    STATUS_CLOUD_FILE_REQUEST_TIMEOUT = 0xc000cf1f,
    STATUS_CLOUD_FILE_DEHYDRATION_DISALLOWED = 0xc000cf20,
    STATUS_FILE_SNAP_IN_PROGRESS = 0xc000f500,
    STATUS_FILE_SNAP_USER_SECTION_NOT_SUPPORTED = 0xc000f501,
    STATUS_FILE_SNAP_MODIFY_NOT_SUPPORTED = 0xc000f502,
    STATUS_FILE_SNAP_IO_NOT_COORDINATED = 0xc000f503,
    STATUS_FILE_SNAP_UNEXPECTED_ERROR = 0xc000f504,
    STATUS_FILE_SNAP_INVALID_PARAMETER = 0xc000f505,
    DBG_NO_STATE_CHANGE = 0xc0010001,
    DBG_APP_NOT_IDLE = 0xc0010002,
    RPC_NT_INVALID_STRING_BINDING = 0xc0020001,
    RPC_NT_WRONG_KIND_OF_BINDING = 0xc0020002,
    RPC_NT_INVALID_BINDING = 0xc0020003,
    RPC_NT_PROTSEQ_NOT_SUPPORTED = 0xc0020004,
    RPC_NT_INVALID_RPC_PROTSEQ = 0xc0020005,
    RPC_NT_INVALID_STRING_UUID = 0xc0020006,
    RPC_NT_INVALID_ENDPOINT_FORMAT = 0xc0020007,
    RPC_NT_INVALID_NET_ADDR = 0xc0020008,
    RPC_NT_NO_ENDPOINT_FOUND = 0xc0020009,
    RPC_NT_INVALID_TIMEOUT = 0xc002000a,
    RPC_NT_OBJECT_NOT_FOUND = 0xc002000b,
    RPC_NT_ALREADY_REGISTERED = 0xc002000c,
    RPC_NT_TYPE_ALREADY_REGISTERED = 0xc002000d,
    RPC_NT_ALREADY_LISTENING = 0xc002000e,
    RPC_NT_NO_PROTSEQS_REGISTERED = 0xc002000f,
    RPC_NT_NOT_LISTENING = 0xc0020010,
    RPC_NT_UNKNOWN_MGR_TYPE = 0xc0020011,
    RPC_NT_UNKNOWN_IF = 0xc0020012,
    RPC_NT_NO_BINDINGS = 0xc0020013,
    RPC_NT_NO_PROTSEQS = 0xc0020014,
    RPC_NT_CANT_CREATE_ENDPOINT = 0xc0020015,
    RPC_NT_OUT_OF_RESOURCES = 0xc0020016,
    RPC_NT_SERVER_UNAVAILABLE = 0xc0020017,
    RPC_NT_SERVER_TOO_BUSY = 0xc0020018,
    RPC_NT_INVALID_NETWORK_OPTIONS = 0xc0020019,
    RPC_NT_NO_CALL_ACTIVE = 0xc002001a,
    RPC_NT_CALL_FAILED = 0xc002001b,
    RPC_NT_CALL_FAILED_DNE = 0xc002001c,
    RPC_NT_PROTOCOL_ERROR = 0xc002001d,
    RPC_NT_UNSUPPORTED_TRANS_SYN = 0xc002001f,
    RPC_NT_UNSUPPORTED_TYPE = 0xc0020021,
    RPC_NT_INVALID_TAG = 0xc0020022,
    RPC_NT_INVALID_BOUND = 0xc0020023,
    RPC_NT_NO_ENTRY_NAME = 0xc0020024,
    RPC_NT_INVALID_NAME_SYNTAX = 0xc0020025,
    RPC_NT_UNSUPPORTED_NAME_SYNTAX = 0xc0020026,
    RPC_NT_UUID_NO_ADDRESS = 0xc0020028,
    RPC_NT_DUPLICATE_ENDPOINT = 0xc0020029,
    RPC_NT_UNKNOWN_AUTHN_TYPE = 0xc002002a,
    RPC_NT_MAX_CALLS_TOO_SMALL = 0xc002002b,
    RPC_NT_STRING_TOO_LONG = 0xc002002c,
    RPC_NT_PROTSEQ_NOT_FOUND = 0xc002002d,
    RPC_NT_PROCNUM_OUT_OF_RANGE = 0xc002002e,
    RPC_NT_BINDING_HAS_NO_AUTH = 0xc002002f,
    RPC_NT_UNKNOWN_AUTHN_SERVICE = 0xc0020030,
    RPC_NT_UNKNOWN_AUTHN_LEVEL = 0xc0020031,
    RPC_NT_INVALID_AUTH_IDENTITY = 0xc0020032,
    RPC_NT_UNKNOWN_AUTHZ_SERVICE = 0xc0020033,
    EPT_NT_INVALID_ENTRY = 0xc0020034,
    EPT_NT_CANT_PERFORM_OP = 0xc0020035,
    EPT_NT_NOT_REGISTERED = 0xc0020036,
    RPC_NT_NOTHING_TO_EXPORT = 0xc0020037,
    RPC_NT_INCOMPLETE_NAME = 0xc0020038,
    RPC_NT_INVALID_VERS_OPTION = 0xc0020039,
    RPC_NT_NO_MORE_MEMBERS = 0xc002003a,
    RPC_NT_NOT_ALL_OBJS_UNEXPORTED = 0xc002003b,
    RPC_NT_INTERFACE_NOT_FOUND = 0xc002003c,
    RPC_NT_ENTRY_ALREADY_EXISTS = 0xc002003d,
    RPC_NT_ENTRY_NOT_FOUND = 0xc002003e,
    RPC_NT_NAME_SERVICE_UNAVAILABLE = 0xc002003f,
    RPC_NT_INVALID_NAF_ID = 0xc0020040,
    RPC_NT_CANNOT_SUPPORT = 0xc0020041,
    RPC_NT_NO_CONTEXT_AVAILABLE = 0xc0020042,
    RPC_NT_INTERNAL_ERROR = 0xc0020043,
    RPC_NT_ZERO_DIVIDE = 0xc0020044,
    RPC_NT_ADDRESS_ERROR = 0xc0020045,
    RPC_NT_FP_DIV_ZERO = 0xc0020046,
    RPC_NT_FP_UNDERFLOW = 0xc0020047,
    RPC_NT_FP_OVERFLOW = 0xc0020048,
    RPC_NT_CALL_IN_PROGRESS = 0xc0020049,
    RPC_NT_NO_MORE_BINDINGS = 0xc002004a,
    RPC_NT_GROUP_MEMBER_NOT_FOUND = 0xc002004b,
    EPT_NT_CANT_CREATE = 0xc002004c,
    RPC_NT_INVALID_OBJECT = 0xc002004d,
    RPC_NT_NO_INTERFACES = 0xc002004f,
    RPC_NT_CALL_CANCELLED = 0xc0020050,
    RPC_NT_BINDING_INCOMPLETE = 0xc0020051,
    RPC_NT_COMM_FAILURE = 0xc0020052,
    RPC_NT_UNSUPPORTED_AUTHN_LEVEL = 0xc0020053,
    RPC_NT_NO_PRINC_NAME = 0xc0020054,
    RPC_NT_NOT_RPC_ERROR = 0xc0020055,
    RPC_NT_SEC_PKG_ERROR = 0xc0020057,
    RPC_NT_NOT_CANCELLED = 0xc0020058,
    RPC_NT_INVALID_ASYNC_HANDLE = 0xc0020062,
    RPC_NT_INVALID_ASYNC_CALL = 0xc0020063,
    RPC_NT_PROXY_ACCESS_DENIED = 0xc0020064,
    RPC_NT_COOKIE_AUTH_FAILED = 0xc0020065,
    RPC_NT_NO_MORE_ENTRIES = 0xc0030001,
    RPC_NT_SS_CHAR_TRANS_OPEN_FAIL = 0xc0030002,
    RPC_NT_SS_CHAR_TRANS_SHORT_FILE = 0xc0030003,
    RPC_NT_SS_IN_NULL_CONTEXT = 0xc0030004,
    RPC_NT_SS_CONTEXT_MISMATCH = 0xc0030005,
    RPC_NT_SS_CONTEXT_DAMAGED = 0xc0030006,
    RPC_NT_SS_HANDLES_MISMATCH = 0xc0030007,
    RPC_NT_SS_CANNOT_GET_CALL_HANDLE = 0xc0030008,
    RPC_NT_NULL_REF_POINTER = 0xc0030009,
    RPC_NT_ENUM_VALUE_OUT_OF_RANGE = 0xc003000a,
    RPC_NT_BYTE_COUNT_TOO_SMALL = 0xc003000b,
    RPC_NT_BAD_STUB_DATA = 0xc003000c,
    RPC_NT_INVALID_ES_ACTION = 0xc0030059,
    RPC_NT_WRONG_ES_VERSION = 0xc003005a,
    RPC_NT_WRONG_STUB_VERSION = 0xc003005b,
    RPC_NT_INVALID_PIPE_OBJECT = 0xc003005c,
    RPC_NT_INVALID_PIPE_OPERATION = 0xc003005d,
    RPC_NT_WRONG_PIPE_VERSION = 0xc003005e,
    RPC_NT_PIPE_CLOSED = 0xc003005f,
    RPC_NT_PIPE_DISCIPLINE_ERROR = 0xc0030060,
    RPC_NT_PIPE_EMPTY = 0xc0030061,
    STATUS_PNP_BAD_MPS_TABLE = 0xc0040035,
    STATUS_PNP_TRANSLATION_FAILED = 0xc0040036,
    STATUS_PNP_IRQ_TRANSLATION_FAILED = 0xc0040037,
    STATUS_PNP_INVALID_ID = 0xc0040038,
    STATUS_IO_REISSUE_AS_CACHED = 0xc0040039,
    STATUS_CTX_WINSTATION_NAME_INVALID = 0xc00a0001,
    STATUS_CTX_INVALID_PD = 0xc00a0002,
    STATUS_CTX_PD_NOT_FOUND = 0xc00a0003,
    STATUS_CTX_CLOSE_PENDING = 0xc00a0006,
    STATUS_CTX_NO_OUTBUF = 0xc00a0007,
    STATUS_CTX_MODEM_INF_NOT_FOUND = 0xc00a0008,
    STATUS_CTX_INVALID_MODEMNAME = 0xc00a0009,
    STATUS_CTX_RESPONSE_ERROR = 0xc00a000a,
    STATUS_CTX_MODEM_RESPONSE_TIMEOUT = 0xc00a000b,
    STATUS_CTX_MODEM_RESPONSE_NO_CARRIER = 0xc00a000c,
    STATUS_CTX_MODEM_RESPONSE_NO_DIALTONE = 0xc00a000d,
    STATUS_CTX_MODEM_RESPONSE_BUSY = 0xc00a000e,
    STATUS_CTX_MODEM_RESPONSE_VOICE = 0xc00a000f,
    STATUS_CTX_TD_ERROR = 0xc00a0010,
    STATUS_CTX_LICENSE_CLIENT_INVALID = 0xc00a0012,
    STATUS_CTX_LICENSE_NOT_AVAILABLE = 0xc00a0013,
    STATUS_CTX_LICENSE_EXPIRED = 0xc00a0014,
    STATUS_CTX_WINSTATION_NOT_FOUND = 0xc00a0015,
    STATUS_CTX_WINSTATION_NAME_COLLISION = 0xc00a0016,
    STATUS_CTX_WINSTATION_BUSY = 0xc00a0017,
    STATUS_CTX_BAD_VIDEO_MODE = 0xc00a0018,
    STATUS_CTX_GRAPHICS_INVALID = 0xc00a0022,
    STATUS_CTX_NOT_CONSOLE = 0xc00a0024,
    STATUS_CTX_CLIENT_QUERY_TIMEOUT = 0xc00a0026,
    STATUS_CTX_CONSOLE_DISCONNECT = 0xc00a0027,
    STATUS_CTX_CONSOLE_CONNECT = 0xc00a0028,
    STATUS_CTX_SHADOW_DENIED = 0xc00a002a,
    STATUS_CTX_WINSTATION_ACCESS_DENIED = 0xc00a002b,
    STATUS_CTX_INVALID_WD = 0xc00a002e,
    STATUS_CTX_WD_NOT_FOUND = 0xc00a002f,
    STATUS_CTX_SHADOW_INVALID = 0xc00a0030,
    STATUS_CTX_SHADOW_DISABLED = 0xc00a0031,
    STATUS_RDP_PROTOCOL_ERROR = 0xc00a0032,
    STATUS_CTX_CLIENT_LICENSE_NOT_SET = 0xc00a0033,
    STATUS_CTX_CLIENT_LICENSE_IN_USE = 0xc00a0034,
    STATUS_CTX_SHADOW_ENDED_BY_MODE_CHANGE = 0xc00a0035,
    STATUS_CTX_SHADOW_NOT_RUNNING = 0xc00a0036,
    STATUS_CTX_LOGON_DISABLED = 0xc00a0037,
    STATUS_CTX_SECURITY_LAYER_ERROR = 0xc00a0038,
    STATUS_TS_INCOMPATIBLE_SESSIONS = 0xc00a0039,
    STATUS_TS_VIDEO_SUBSYSTEM_ERROR = 0xc00a003a,
    STATUS_MUI_FILE_NOT_FOUND = 0xc00b0001,
    STATUS_MUI_INVALID_FILE = 0xc00b0002,
    STATUS_MUI_INVALID_RC_CONFIG = 0xc00b0003,
    STATUS_MUI_INVALID_LOCALE_NAME = 0xc00b0004,
    STATUS_MUI_INVALID_ULTIMATEFALLBACK_NAME = 0xc00b0005,
    STATUS_MUI_FILE_NOT_LOADED = 0xc00b0006,
    STATUS_RESOURCE_ENUM_USER_STOP = 0xc00b0007,
    STATUS_CLUSTER_INVALID_NODE = 0xc0130001,
    STATUS_CLUSTER_NODE_EXISTS = 0xc0130002,
    STATUS_CLUSTER_JOIN_IN_PROGRESS = 0xc0130003,
    STATUS_CLUSTER_NODE_NOT_FOUND = 0xc0130004,
    STATUS_CLUSTER_LOCAL_NODE_NOT_FOUND = 0xc0130005,
    STATUS_CLUSTER_NETWORK_EXISTS = 0xc0130006,
    STATUS_CLUSTER_NETWORK_NOT_FOUND = 0xc0130007,
    STATUS_CLUSTER_NETINTERFACE_EXISTS = 0xc0130008,
    STATUS_CLUSTER_NETINTERFACE_NOT_FOUND = 0xc0130009,
    STATUS_CLUSTER_INVALID_REQUEST = 0xc013000a,
    STATUS_CLUSTER_INVALID_NETWORK_PROVIDER = 0xc013000b,
    STATUS_CLUSTER_NODE_DOWN = 0xc013000c,
    STATUS_CLUSTER_NODE_UNREACHABLE = 0xc013000d,
    STATUS_CLUSTER_NODE_NOT_MEMBER = 0xc013000e,
    STATUS_CLUSTER_JOIN_NOT_IN_PROGRESS = 0xc013000f,
    STATUS_CLUSTER_INVALID_NETWORK = 0xc0130010,
    STATUS_CLUSTER_NO_NET_ADAPTERS = 0xc0130011,
    STATUS_CLUSTER_NODE_UP = 0xc0130012,
    STATUS_CLUSTER_NODE_PAUSED = 0xc0130013,
    STATUS_CLUSTER_NODE_NOT_PAUSED = 0xc0130014,
    STATUS_CLUSTER_NO_SECURITY_CONTEXT = 0xc0130015,
    STATUS_CLUSTER_NETWORK_NOT_INTERNAL = 0xc0130016,
    STATUS_CLUSTER_POISONED = 0xc0130017,
    STATUS_CLUSTER_NON_CSV_PATH = 0xc0130018,
    STATUS_CLUSTER_CSV_VOLUME_NOT_LOCAL = 0xc0130019,
    STATUS_CLUSTER_CSV_READ_OPLOCK_BREAK_IN_PROGRESS = 0xc0130020,
    STATUS_CLUSTER_CSV_AUTO_PAUSE_ERROR = 0xc0130021,
    STATUS_CLUSTER_CSV_REDIRECTED = 0xc0130022,
    STATUS_CLUSTER_CSV_NOT_REDIRECTED = 0xc0130023,
    STATUS_CLUSTER_CSV_VOLUME_DRAINING = 0xc0130024,
    STATUS_CLUSTER_CSV_SNAPSHOT_CREATION_IN_PROGRESS = 0xc0130025,
    STATUS_CLUSTER_CSV_VOLUME_DRAINING_SUCCEEDED_DOWNLEVEL = 0xc0130026,
    STATUS_CLUSTER_CSV_NO_SNAPSHOTS = 0xc0130027,
    STATUS_CSV_IO_PAUSE_TIMEOUT = 0xc0130028,
    STATUS_CLUSTER_CSV_INVALID_HANDLE = 0xc0130029,
    STATUS_CLUSTER_CSV_SUPPORTED_ONLY_ON_COORDINATOR = 0xc0130030,
    STATUS_CLUSTER_CAM_TICKET_REPLAY_DETECTED = 0xc0130031,
    STATUS_ACPI_INVALID_OPCODE = 0xc0140001,
    STATUS_ACPI_STACK_OVERFLOW = 0xc0140002,
    STATUS_ACPI_ASSERT_FAILED = 0xc0140003,
    STATUS_ACPI_INVALID_INDEX = 0xc0140004,
    STATUS_ACPI_INVALID_ARGUMENT = 0xc0140005,
    STATUS_ACPI_FATAL = 0xc0140006,
    STATUS_ACPI_INVALID_SUPERNAME = 0xc0140007,
    STATUS_ACPI_INVALID_ARGTYPE = 0xc0140008,
    STATUS_ACPI_INVALID_OBJTYPE = 0xc0140009,
    STATUS_ACPI_INVALID_TARGETTYPE = 0xc014000a,
    STATUS_ACPI_INCORRECT_ARGUMENT_COUNT = 0xc014000b,
    STATUS_ACPI_ADDRESS_NOT_MAPPED = 0xc014000c,
    STATUS_ACPI_INVALID_EVENTTYPE = 0xc014000d,
    STATUS_ACPI_HANDLER_COLLISION = 0xc014000e,
    STATUS_ACPI_INVALID_DATA = 0xc014000f,
    STATUS_ACPI_INVALID_REGION = 0xc0140010,
    STATUS_ACPI_INVALID_ACCESS_SIZE = 0xc0140011,
    STATUS_ACPI_ACQUIRE_GLOBAL_LOCK = 0xc0140012,
    STATUS_ACPI_ALREADY_INITIALIZED = 0xc0140013,
    STATUS_ACPI_NOT_INITIALIZED = 0xc0140014,
    STATUS_ACPI_INVALID_MUTEX_LEVEL = 0xc0140015,
    STATUS_ACPI_MUTEX_NOT_OWNED = 0xc0140016,
    STATUS_ACPI_MUTEX_NOT_OWNER = 0xc0140017,
    STATUS_ACPI_RS_ACCESS = 0xc0140018,
    STATUS_ACPI_INVALID_TABLE = 0xc0140019,
    STATUS_ACPI_REG_HANDLER_FAILED = 0xc0140020,
    STATUS_ACPI_POWER_REQUEST_FAILED = 0xc0140021,
    STATUS_SXS_SECTION_NOT_FOUND = 0xc0150001,
    STATUS_SXS_CANT_GEN_ACTCTX = 0xc0150002,
    STATUS_SXS_INVALID_ACTCTXDATA_FORMAT = 0xc0150003,
    STATUS_SXS_ASSEMBLY_NOT_FOUND = 0xc0150004,
    STATUS_SXS_MANIFEST_FORMAT_ERROR = 0xc0150005,
    STATUS_SXS_MANIFEST_PARSE_ERROR = 0xc0150006,
    STATUS_SXS_ACTIVATION_CONTEXT_DISABLED = 0xc0150007,
    STATUS_SXS_KEY_NOT_FOUND = 0xc0150008,
    STATUS_SXS_VERSION_CONFLICT = 0xc0150009,
    STATUS_SXS_WRONG_SECTION_TYPE = 0xc015000a,
    STATUS_SXS_THREAD_QUERIES_DISABLED = 0xc015000b,
    STATUS_SXS_ASSEMBLY_MISSING = 0xc015000c,
    STATUS_SXS_PROCESS_DEFAULT_ALREADY_SET = 0xc015000e,
    STATUS_SXS_EARLY_DEACTIVATION = 0xc015000f,
    STATUS_SXS_INVALID_DEACTIVATION = 0xc0150010,
    STATUS_SXS_MULTIPLE_DEACTIVATION = 0xc0150011,
    STATUS_SXS_SYSTEM_DEFAULT_ACTIVATION_CONTEXT_EMPTY = 0xc0150012,
    STATUS_SXS_PROCESS_TERMINATION_REQUESTED = 0xc0150013,
    STATUS_SXS_CORRUPT_ACTIVATION_STACK = 0xc0150014,
    STATUS_SXS_CORRUPTION = 0xc0150015,
    STATUS_SXS_INVALID_IDENTITY_ATTRIBUTE_VALUE = 0xc0150016,
    STATUS_SXS_INVALID_IDENTITY_ATTRIBUTE_NAME = 0xc0150017,
    STATUS_SXS_IDENTITY_DUPLICATE_ATTRIBUTE = 0xc0150018,
    STATUS_SXS_IDENTITY_PARSE_ERROR = 0xc0150019,
    STATUS_SXS_COMPONENT_STORE_CORRUPT = 0xc015001a,
    STATUS_SXS_FILE_HASH_MISMATCH = 0xc015001b,
    STATUS_SXS_MANIFEST_IDENTITY_SAME_BUT_CONTENTS_DIFFERENT = 0xc015001c,
    STATUS_SXS_IDENTITIES_DIFFERENT = 0xc015001d,
    STATUS_SXS_ASSEMBLY_IS_NOT_A_DEPLOYMENT = 0xc015001e,
    STATUS_SXS_FILE_NOT_PART_OF_ASSEMBLY = 0xc015001f,
    STATUS_ADVANCED_INSTALLER_FAILED = 0xc0150020,
    STATUS_XML_ENCODING_MISMATCH = 0xc0150021,
    STATUS_SXS_MANIFEST_TOO_BIG = 0xc0150022,
    STATUS_SXS_SETTING_NOT_REGISTERED = 0xc0150023,
    STATUS_SXS_TRANSACTION_CLOSURE_INCOMPLETE = 0xc0150024,
    STATUS_SMI_PRIMITIVE_INSTALLER_FAILED = 0xc0150025,
    STATUS_GENERIC_COMMAND_FAILED = 0xc0150026,
    STATUS_SXS_FILE_HASH_MISSING = 0xc0150027,
    STATUS_TRANSACTIONAL_CONFLICT = 0xc0190001,
    STATUS_INVALID_TRANSACTION = 0xc0190002,
    STATUS_TRANSACTION_NOT_ACTIVE = 0xc0190003,
    STATUS_TM_INITIALIZATION_FAILED = 0xc0190004,
    STATUS_RM_NOT_ACTIVE = 0xc0190005,
    STATUS_RM_METADATA_CORRUPT = 0xc0190006,
    STATUS_TRANSACTION_NOT_JOINED = 0xc0190007,
    STATUS_DIRECTORY_NOT_RM = 0xc0190008,
    STATUS_TRANSACTIONS_UNSUPPORTED_REMOTE = 0xc019000a,
    STATUS_LOG_RESIZE_INVALID_SIZE = 0xc019000b,
    STATUS_REMOTE_FILE_VERSION_MISMATCH = 0xc019000c,
    STATUS_CRM_PROTOCOL_ALREADY_EXISTS = 0xc019000f,
    STATUS_TRANSACTION_PROPAGATION_FAILED = 0xc0190010,
    STATUS_CRM_PROTOCOL_NOT_FOUND = 0xc0190011,
    STATUS_TRANSACTION_SUPERIOR_EXISTS = 0xc0190012,
    STATUS_TRANSACTION_REQUEST_NOT_VALID = 0xc0190013,
    STATUS_TRANSACTION_NOT_REQUESTED = 0xc0190014,
    STATUS_TRANSACTION_ALREADY_ABORTED = 0xc0190015,
    STATUS_TRANSACTION_ALREADY_COMMITTED = 0xc0190016,
    STATUS_TRANSACTION_INVALID_MARSHALL_BUFFER = 0xc0190017,
    STATUS_CURRENT_TRANSACTION_NOT_VALID = 0xc0190018,
    STATUS_LOG_GROWTH_FAILED = 0xc0190019,
    STATUS_OBJECT_NO_LONGER_EXISTS = 0xc0190021,
    STATUS_STREAM_MINIVERSION_NOT_FOUND = 0xc0190022,
    STATUS_STREAM_MINIVERSION_NOT_VALID = 0xc0190023,
    STATUS_MINIVERSION_INACCESSIBLE_FROM_SPECIFIED_TRANSACTION = 0xc0190024,
    STATUS_CANT_OPEN_MINIVERSION_WITH_MODIFY_INTENT = 0xc0190025,
    STATUS_CANT_CREATE_MORE_STREAM_MINIVERSIONS = 0xc0190026,
    STATUS_HANDLE_NO_LONGER_VALID = 0xc0190028,
    STATUS_LOG_CORRUPTION_DETECTED = 0xc0190030,
    STATUS_RM_DISCONNECTED = 0xc0190032,
    STATUS_ENLISTMENT_NOT_SUPERIOR = 0xc0190033,
    STATUS_FILE_IDENTITY_NOT_PERSISTENT = 0xc0190036,
    STATUS_CANT_BREAK_TRANSACTIONAL_DEPENDENCY = 0xc0190037,
    STATUS_CANT_CROSS_RM_BOUNDARY = 0xc0190038,
    STATUS_TXF_DIR_NOT_EMPTY = 0xc0190039,
    STATUS_INDOUBT_TRANSACTIONS_EXIST = 0xc019003a,
    STATUS_TM_VOLATILE = 0xc019003b,
    STATUS_ROLLBACK_TIMER_EXPIRED = 0xc019003c,
    STATUS_TXF_ATTRIBUTE_CORRUPT = 0xc019003d,
    STATUS_EFS_NOT_ALLOWED_IN_TRANSACTION = 0xc019003e,
    STATUS_TRANSACTIONAL_OPEN_NOT_ALLOWED = 0xc019003f,
    STATUS_TRANSACTED_MAPPING_UNSUPPORTED_REMOTE = 0xc0190040,
    STATUS_TRANSACTION_REQUIRED_PROMOTION = 0xc0190043,
    STATUS_CANNOT_EXECUTE_FILE_IN_TRANSACTION = 0xc0190044,
    STATUS_TRANSACTIONS_NOT_FROZEN = 0xc0190045,
    STATUS_TRANSACTION_FREEZE_IN_PROGRESS = 0xc0190046,
    STATUS_NOT_SNAPSHOT_VOLUME = 0xc0190047,
    STATUS_NO_SAVEPOINT_WITH_OPEN_FILES = 0xc0190048,
    STATUS_SPARSE_NOT_ALLOWED_IN_TRANSACTION = 0xc0190049,
    STATUS_TM_IDENTITY_MISMATCH = 0xc019004a,
    STATUS_FLOATED_SECTION = 0xc019004b,
    STATUS_CANNOT_ACCEPT_TRANSACTED_WORK = 0xc019004c,
    STATUS_CANNOT_ABORT_TRANSACTIONS = 0xc019004d,
    STATUS_TRANSACTION_NOT_FOUND = 0xc019004e,
    STATUS_RESOURCEMANAGER_NOT_FOUND = 0xc019004f,
    STATUS_ENLISTMENT_NOT_FOUND = 0xc0190050,
    STATUS_TRANSACTIONMANAGER_NOT_FOUND = 0xc0190051,
    STATUS_TRANSACTIONMANAGER_NOT_ONLINE = 0xc0190052,
    STATUS_TRANSACTIONMANAGER_RECOVERY_NAME_COLLISION = 0xc0190053,
    STATUS_TRANSACTION_NOT_ROOT = 0xc0190054,
    STATUS_TRANSACTION_OBJECT_EXPIRED = 0xc0190055,
    STATUS_COMPRESSION_NOT_ALLOWED_IN_TRANSACTION = 0xc0190056,
    STATUS_TRANSACTION_RESPONSE_NOT_ENLISTED = 0xc0190057,
    STATUS_TRANSACTION_RECORD_TOO_LONG = 0xc0190058,
    STATUS_NO_LINK_TRACKING_IN_TRANSACTION = 0xc0190059,
    STATUS_OPERATION_NOT_SUPPORTED_IN_TRANSACTION = 0xc019005a,
    STATUS_TRANSACTION_INTEGRITY_VIOLATED = 0xc019005b,
    STATUS_TRANSACTIONMANAGER_IDENTITY_MISMATCH = 0xc019005c,
    STATUS_RM_CANNOT_BE_FROZEN_FOR_SNAPSHOT = 0xc019005d,
    STATUS_TRANSACTION_MUST_WRITETHROUGH = 0xc019005e,
    STATUS_TRANSACTION_NO_SUPERIOR = 0xc019005f,
    STATUS_EXPIRED_HANDLE = 0xc0190060,
    STATUS_TRANSACTION_NOT_ENLISTED = 0xc0190061,
    STATUS_LOG_SECTOR_INVALID = 0xc01a0001,
    STATUS_LOG_SECTOR_PARITY_INVALID = 0xc01a0002,
    STATUS_LOG_SECTOR_REMAPPED = 0xc01a0003,
    STATUS_LOG_BLOCK_INCOMPLETE = 0xc01a0004,
    STATUS_LOG_INVALID_RANGE = 0xc01a0005,
    STATUS_LOG_BLOCKS_EXHAUSTED = 0xc01a0006,
    STATUS_LOG_READ_CONTEXT_INVALID = 0xc01a0007,
    STATUS_LOG_RESTART_INVALID = 0xc01a0008,
    STATUS_LOG_BLOCK_VERSION = 0xc01a0009,
    STATUS_LOG_BLOCK_INVALID = 0xc01a000a,
    STATUS_LOG_READ_MODE_INVALID = 0xc01a000b,
    STATUS_LOG_METADATA_CORRUPT = 0xc01a000d,
    STATUS_LOG_METADATA_INVALID = 0xc01a000e,
    STATUS_LOG_METADATA_INCONSISTENT = 0xc01a000f,
    STATUS_LOG_RESERVATION_INVALID = 0xc01a0010,
    STATUS_LOG_CANT_DELETE = 0xc01a0011,
    STATUS_LOG_CONTAINER_LIMIT_EXCEEDED = 0xc01a0012,
    STATUS_LOG_START_OF_LOG = 0xc01a0013,
    STATUS_LOG_POLICY_ALREADY_INSTALLED = 0xc01a0014,
    STATUS_LOG_POLICY_NOT_INSTALLED = 0xc01a0015,
    STATUS_LOG_POLICY_INVALID = 0xc01a0016,
    STATUS_LOG_POLICY_CONFLICT = 0xc01a0017,
    STATUS_LOG_PINNED_ARCHIVE_TAIL = 0xc01a0018,
    STATUS_LOG_RECORD_NONEXISTENT = 0xc01a0019,
    STATUS_LOG_RECORDS_RESERVED_INVALID = 0xc01a001a,
    STATUS_LOG_SPACE_RESERVED_INVALID = 0xc01a001b,
    STATUS_LOG_TAIL_INVALID = 0xc01a001c,
    STATUS_LOG_FULL = 0xc01a001d,
    STATUS_LOG_MULTIPLEXED = 0xc01a001e,
    STATUS_LOG_DEDICATED = 0xc01a001f,
    STATUS_LOG_ARCHIVE_NOT_IN_PROGRESS = 0xc01a0020,
    STATUS_LOG_ARCHIVE_IN_PROGRESS = 0xc01a0021,
    STATUS_LOG_EPHEMERAL = 0xc01a0022,
    STATUS_LOG_NOT_ENOUGH_CONTAINERS = 0xc01a0023,
    STATUS_LOG_CLIENT_ALREADY_REGISTERED = 0xc01a0024,
    STATUS_LOG_CLIENT_NOT_REGISTERED = 0xc01a0025,
    STATUS_LOG_FULL_HANDLER_IN_PROGRESS = 0xc01a0026,
    STATUS_LOG_CONTAINER_READ_FAILED = 0xc01a0027,
    STATUS_LOG_CONTAINER_WRITE_FAILED = 0xc01a0028,
    STATUS_LOG_CONTAINER_OPEN_FAILED = 0xc01a0029,
    STATUS_LOG_CONTAINER_STATE_INVALID = 0xc01a002a,
    STATUS_LOG_STATE_INVALID = 0xc01a002b,
    STATUS_LOG_PINNED = 0xc01a002c,
    STATUS_LOG_METADATA_FLUSH_FAILED = 0xc01a002d,
    STATUS_LOG_INCONSISTENT_SECURITY = 0xc01a002e,
    STATUS_LOG_APPENDED_FLUSH_FAILED = 0xc01a002f,
    STATUS_LOG_PINNED_RESERVATION = 0xc01a0030,
    STATUS_VIDEO_HUNG_DISPLAY_DRIVER_THREAD = 0xc01b00ea,
    STATUS_FLT_NO_HANDLER_DEFINED = 0xc01c0001,
    STATUS_FLT_CONTEXT_ALREADY_DEFINED = 0xc01c0002,
    STATUS_FLT_INVALID_ASYNCHRONOUS_REQUEST = 0xc01c0003,
    STATUS_FLT_DISALLOW_FAST_IO = 0xc01c0004,
    STATUS_FLT_INVALID_NAME_REQUEST = 0xc01c0005,
    STATUS_FLT_NOT_SAFE_TO_POST_OPERATION = 0xc01c0006,
    STATUS_FLT_NOT_INITIALIZED = 0xc01c0007,
    STATUS_FLT_FILTER_NOT_READY = 0xc01c0008,
    STATUS_FLT_POST_OPERATION_CLEANUP = 0xc01c0009,
    STATUS_FLT_INTERNAL_ERROR = 0xc01c000a,
    STATUS_FLT_DELETING_OBJECT = 0xc01c000b,
    STATUS_FLT_MUST_BE_NONPAGED_POOL = 0xc01c000c,
    STATUS_FLT_DUPLICATE_ENTRY = 0xc01c000d,
    STATUS_FLT_CBDQ_DISABLED = 0xc01c000e,
    STATUS_FLT_DO_NOT_ATTACH = 0xc01c000f,
    STATUS_FLT_DO_NOT_DETACH = 0xc01c0010,
    STATUS_FLT_INSTANCE_ALTITUDE_COLLISION = 0xc01c0011,
    STATUS_FLT_INSTANCE_NAME_COLLISION = 0xc01c0012,
    STATUS_FLT_FILTER_NOT_FOUND = 0xc01c0013,
    STATUS_FLT_VOLUME_NOT_FOUND = 0xc01c0014,
    STATUS_FLT_INSTANCE_NOT_FOUND = 0xc01c0015,
    STATUS_FLT_CONTEXT_ALLOCATION_NOT_FOUND = 0xc01c0016,
    STATUS_FLT_INVALID_CONTEXT_REGISTRATION = 0xc01c0017,
    STATUS_FLT_NAME_CACHE_MISS = 0xc01c0018,
    STATUS_FLT_NO_DEVICE_OBJECT = 0xc01c0019,
    STATUS_FLT_VOLUME_ALREADY_MOUNTED = 0xc01c001a,
    STATUS_FLT_ALREADY_ENLISTED = 0xc01c001b,
    STATUS_FLT_CONTEXT_ALREADY_LINKED = 0xc01c001c,
    STATUS_FLT_NO_WAITER_FOR_REPLY = 0xc01c0020,
    STATUS_FLT_REGISTRATION_BUSY = 0xc01c0023,
    STATUS_MONITOR_NO_DESCRIPTOR = 0xc01d0001,
    STATUS_MONITOR_UNKNOWN_DESCRIPTOR_FORMAT = 0xc01d0002,
    STATUS_MONITOR_INVALID_DESCRIPTOR_CHECKSUM = 0xc01d0003,
    STATUS_MONITOR_INVALID_STANDARD_TIMING_BLOCK = 0xc01d0004,
    STATUS_MONITOR_WMI_DATABLOCK_REGISTRATION_FAILED = 0xc01d0005,
    STATUS_MONITOR_INVALID_SERIAL_NUMBER_MONDSC_BLOCK = 0xc01d0006,
    STATUS_MONITOR_INVALID_USER_FRIENDLY_MONDSC_BLOCK = 0xc01d0007,
    STATUS_MONITOR_NO_MORE_DESCRIPTOR_DATA = 0xc01d0008,
    STATUS_MONITOR_INVALID_DETAILED_TIMING_BLOCK = 0xc01d0009,
    STATUS_MONITOR_INVALID_MANUFACTURE_DATE = 0xc01d000a,
    STATUS_GRAPHICS_NOT_EXCLUSIVE_MODE_OWNER = 0xc01e0000,
    STATUS_GRAPHICS_INSUFFICIENT_DMA_BUFFER = 0xc01e0001,
    STATUS_GRAPHICS_INVALID_DISPLAY_ADAPTER = 0xc01e0002,
    STATUS_GRAPHICS_ADAPTER_WAS_RESET = 0xc01e0003,
    STATUS_GRAPHICS_INVALID_DRIVER_MODEL = 0xc01e0004,
    STATUS_GRAPHICS_PRESENT_MODE_CHANGED = 0xc01e0005,
    STATUS_GRAPHICS_PRESENT_OCCLUDED = 0xc01e0006,
    STATUS_GRAPHICS_PRESENT_DENIED = 0xc01e0007,
    STATUS_GRAPHICS_CANNOTCOLORCONVERT = 0xc01e0008,
    STATUS_GRAPHICS_DRIVER_MISMATCH = 0xc01e0009,
    STATUS_GRAPHICS_PRESENT_REDIRECTION_DISABLED = 0xc01e000b,
    STATUS_GRAPHICS_PRESENT_UNOCCLUDED = 0xc01e000c,
    STATUS_GRAPHICS_WINDOWDC_NOT_AVAILABLE = 0xc01e000d,
    STATUS_GRAPHICS_WINDOWLESS_PRESENT_DISABLED = 0xc01e000e,
    STATUS_GRAPHICS_PRESENT_INVALID_WINDOW = 0xc01e000f,
    STATUS_GRAPHICS_PRESENT_BUFFER_NOT_BOUND = 0xc01e0010,
    STATUS_GRAPHICS_VAIL_STATE_CHANGED = 0xc01e0011,
    STATUS_GRAPHICS_INDIRECT_DISPLAY_ABANDON_SWAPCHAIN = 0xc01e0012,
    STATUS_GRAPHICS_INDIRECT_DISPLAY_DEVICE_STOPPED = 0xc01e0013,
    STATUS_GRAPHICS_NO_VIDEO_MEMORY = 0xc01e0100,
    STATUS_GRAPHICS_CANT_LOCK_MEMORY = 0xc01e0101,
    STATUS_GRAPHICS_ALLOCATION_BUSY = 0xc01e0102,
    STATUS_GRAPHICS_TOO_MANY_REFERENCES = 0xc01e0103,
    STATUS_GRAPHICS_TRY_AGAIN_LATER = 0xc01e0104,
    STATUS_GRAPHICS_TRY_AGAIN_NOW = 0xc01e0105,
    STATUS_GRAPHICS_ALLOCATION_INVALID = 0xc01e0106,
    STATUS_GRAPHICS_UNSWIZZLING_APERTURE_UNAVAILABLE = 0xc01e0107,
    STATUS_GRAPHICS_UNSWIZZLING_APERTURE_UNSUPPORTED = 0xc01e0108,
    STATUS_GRAPHICS_CANT_EVICT_PINNED_ALLOCATION = 0xc01e0109,
    STATUS_GRAPHICS_INVALID_ALLOCATION_USAGE = 0xc01e0110,
    STATUS_GRAPHICS_CANT_RENDER_LOCKED_ALLOCATION = 0xc01e0111,
    STATUS_GRAPHICS_ALLOCATION_CLOSED = 0xc01e0112,
    STATUS_GRAPHICS_INVALID_ALLOCATION_INSTANCE = 0xc01e0113,
    STATUS_GRAPHICS_INVALID_ALLOCATION_HANDLE = 0xc01e0114,
    STATUS_GRAPHICS_WRONG_ALLOCATION_DEVICE = 0xc01e0115,
    STATUS_GRAPHICS_ALLOCATION_CONTENT_LOST = 0xc01e0116,
    STATUS_GRAPHICS_GPU_EXCEPTION_ON_DEVICE = 0xc01e0200,
    STATUS_GRAPHICS_INVALID_VIDPN_TOPOLOGY = 0xc01e0300,
    STATUS_GRAPHICS_VIDPN_TOPOLOGY_NOT_SUPPORTED = 0xc01e0301,
    STATUS_GRAPHICS_VIDPN_TOPOLOGY_CURRENTLY_NOT_SUPPORTED = 0xc01e0302,
    STATUS_GRAPHICS_INVALID_VIDPN = 0xc01e0303,
    STATUS_GRAPHICS_INVALID_VIDEO_PRESENT_SOURCE = 0xc01e0304,
    STATUS_GRAPHICS_INVALID_VIDEO_PRESENT_TARGET = 0xc01e0305,
    STATUS_GRAPHICS_VIDPN_MODALITY_NOT_SUPPORTED = 0xc01e0306,
    STATUS_GRAPHICS_INVALID_VIDPN_SOURCEMODESET = 0xc01e0308,
    STATUS_GRAPHICS_INVALID_VIDPN_TARGETMODESET = 0xc01e0309,
    STATUS_GRAPHICS_INVALID_FREQUENCY = 0xc01e030a,
    STATUS_GRAPHICS_INVALID_ACTIVE_REGION = 0xc01e030b,
    STATUS_GRAPHICS_INVALID_TOTAL_REGION = 0xc01e030c,
    STATUS_GRAPHICS_INVALID_VIDEO_PRESENT_SOURCE_MODE = 0xc01e0310,
    STATUS_GRAPHICS_INVALID_VIDEO_PRESENT_TARGET_MODE = 0xc01e0311,
    STATUS_GRAPHICS_PINNED_MODE_MUST_REMAIN_IN_SET = 0xc01e0312,
    STATUS_GRAPHICS_PATH_ALREADY_IN_TOPOLOGY = 0xc01e0313,
    STATUS_GRAPHICS_MODE_ALREADY_IN_MODESET = 0xc01e0314,
    STATUS_GRAPHICS_INVALID_VIDEOPRESENTSOURCESET = 0xc01e0315,
    STATUS_GRAPHICS_INVALID_VIDEOPRESENTTARGETSET = 0xc01e0316,
    STATUS_GRAPHICS_SOURCE_ALREADY_IN_SET = 0xc01e0317,
    STATUS_GRAPHICS_TARGET_ALREADY_IN_SET = 0xc01e0318,
    STATUS_GRAPHICS_INVALID_VIDPN_PRESENT_PATH = 0xc01e0319,
    STATUS_GRAPHICS_NO_RECOMMENDED_VIDPN_TOPOLOGY = 0xc01e031a,
    STATUS_GRAPHICS_INVALID_MONITOR_FREQUENCYRANGESET = 0xc01e031b,
    STATUS_GRAPHICS_INVALID_MONITOR_FREQUENCYRANGE = 0xc01e031c,
    STATUS_GRAPHICS_FREQUENCYRANGE_NOT_IN_SET = 0xc01e031d,
    STATUS_GRAPHICS_FREQUENCYRANGE_ALREADY_IN_SET = 0xc01e031f,
    STATUS_GRAPHICS_STALE_MODESET = 0xc01e0320,
    STATUS_GRAPHICS_INVALID_MONITOR_SOURCEMODESET = 0xc01e0321,
    STATUS_GRAPHICS_INVALID_MONITOR_SOURCE_MODE = 0xc01e0322,
    STATUS_GRAPHICS_NO_RECOMMENDED_FUNCTIONAL_VIDPN = 0xc01e0323,
    STATUS_GRAPHICS_MODE_ID_MUST_BE_UNIQUE = 0xc01e0324,
    STATUS_GRAPHICS_EMPTY_ADAPTER_MONITOR_MODE_SUPPORT_INTERSECTION = 0xc01e0325,
    STATUS_GRAPHICS_VIDEO_PRESENT_TARGETS_LESS_THAN_SOURCES = 0xc01e0326,
    STATUS_GRAPHICS_PATH_NOT_IN_TOPOLOGY = 0xc01e0327,
    STATUS_GRAPHICS_ADAPTER_MUST_HAVE_AT_LEAST_ONE_SOURCE = 0xc01e0328,
    STATUS_GRAPHICS_ADAPTER_MUST_HAVE_AT_LEAST_ONE_TARGET = 0xc01e0329,
    STATUS_GRAPHICS_INVALID_MONITORDESCRIPTORSET = 0xc01e032a,
    STATUS_GRAPHICS_INVALID_MONITORDESCRIPTOR = 0xc01e032b,
    STATUS_GRAPHICS_MONITORDESCRIPTOR_NOT_IN_SET = 0xc01e032c,
    STATUS_GRAPHICS_MONITORDESCRIPTOR_ALREADY_IN_SET = 0xc01e032d,
    STATUS_GRAPHICS_MONITORDESCRIPTOR_ID_MUST_BE_UNIQUE = 0xc01e032e,
    STATUS_GRAPHICS_INVALID_VIDPN_TARGET_SUBSET_TYPE = 0xc01e032f,
    STATUS_GRAPHICS_RESOURCES_NOT_RELATED = 0xc01e0330,
    STATUS_GRAPHICS_SOURCE_ID_MUST_BE_UNIQUE = 0xc01e0331,
    STATUS_GRAPHICS_TARGET_ID_MUST_BE_UNIQUE = 0xc01e0332,
    STATUS_GRAPHICS_NO_AVAILABLE_VIDPN_TARGET = 0xc01e0333,
    STATUS_GRAPHICS_MONITOR_COULD_NOT_BE_ASSOCIATED_WITH_ADAPTER = 0xc01e0334,
    STATUS_GRAPHICS_NO_VIDPNMGR = 0xc01e0335,
    STATUS_GRAPHICS_NO_ACTIVE_VIDPN = 0xc01e0336,
    STATUS_GRAPHICS_STALE_VIDPN_TOPOLOGY = 0xc01e0337,
    STATUS_GRAPHICS_MONITOR_NOT_CONNECTED = 0xc01e0338,
    STATUS_GRAPHICS_SOURCE_NOT_IN_TOPOLOGY = 0xc01e0339,
    STATUS_GRAPHICS_INVALID_PRIMARYSURFACE_SIZE = 0xc01e033a,
    STATUS_GRAPHICS_INVALID_VISIBLEREGION_SIZE = 0xc01e033b,
    STATUS_GRAPHICS_INVALID_STRIDE = 0xc01e033c,
    STATUS_GRAPHICS_INVALID_PIXELFORMAT = 0xc01e033d,
    STATUS_GRAPHICS_INVALID_COLORBASIS = 0xc01e033e,
    STATUS_GRAPHICS_INVALID_PIXELVALUEACCESSMODE = 0xc01e033f,
    STATUS_GRAPHICS_TARGET_NOT_IN_TOPOLOGY = 0xc01e0340,
    STATUS_GRAPHICS_NO_DISPLAY_MODE_MANAGEMENT_SUPPORT = 0xc01e0341,
    STATUS_GRAPHICS_VIDPN_SOURCE_IN_USE = 0xc01e0342,
    STATUS_GRAPHICS_CANT_ACCESS_ACTIVE_VIDPN = 0xc01e0343,
    STATUS_GRAPHICS_INVALID_PATH_IMPORTANCE_ORDINAL = 0xc01e0344,
    STATUS_GRAPHICS_INVALID_PATH_CONTENT_GEOMETRY_TRANSFORMATION = 0xc01e0345,
    STATUS_GRAPHICS_PATH_CONTENT_GEOMETRY_TRANSFORMATION_NOT_SUPPORTED = 0xc01e0346,
    STATUS_GRAPHICS_INVALID_GAMMA_RAMP = 0xc01e0347,
    STATUS_GRAPHICS_GAMMA_RAMP_NOT_SUPPORTED = 0xc01e0348,
    STATUS_GRAPHICS_MULTISAMPLING_NOT_SUPPORTED = 0xc01e0349,
    STATUS_GRAPHICS_MODE_NOT_IN_MODESET = 0xc01e034a,
    STATUS_GRAPHICS_INVALID_VIDPN_TOPOLOGY_RECOMMENDATION_REASON = 0xc01e034d,
    STATUS_GRAPHICS_INVALID_PATH_CONTENT_TYPE = 0xc01e034e,
    STATUS_GRAPHICS_INVALID_COPYPROTECTION_TYPE = 0xc01e034f,
    STATUS_GRAPHICS_UNASSIGNED_MODESET_ALREADY_EXISTS = 0xc01e0350,
    STATUS_GRAPHICS_INVALID_SCANLINE_ORDERING = 0xc01e0352,
    STATUS_GRAPHICS_TOPOLOGY_CHANGES_NOT_ALLOWED = 0xc01e0353,
    STATUS_GRAPHICS_NO_AVAILABLE_IMPORTANCE_ORDINALS = 0xc01e0354,
    STATUS_GRAPHICS_INCOMPATIBLE_PRIVATE_FORMAT = 0xc01e0355,
    STATUS_GRAPHICS_INVALID_MODE_PRUNING_ALGORITHM = 0xc01e0356,
    STATUS_GRAPHICS_INVALID_MONITOR_CAPABILITY_ORIGIN = 0xc01e0357,
    STATUS_GRAPHICS_INVALID_MONITOR_FREQUENCYRANGE_CONSTRAINT = 0xc01e0358,
    STATUS_GRAPHICS_MAX_NUM_PATHS_REACHED = 0xc01e0359,
    STATUS_GRAPHICS_CANCEL_VIDPN_TOPOLOGY_AUGMENTATION = 0xc01e035a,
    STATUS_GRAPHICS_INVALID_CLIENT_TYPE = 0xc01e035b,
    STATUS_GRAPHICS_CLIENTVIDPN_NOT_SET = 0xc01e035c,
    STATUS_GRAPHICS_SPECIFIED_CHILD_ALREADY_CONNECTED = 0xc01e0400,
    STATUS_GRAPHICS_CHILD_DESCRIPTOR_NOT_SUPPORTED = 0xc01e0401,
    STATUS_GRAPHICS_NOT_A_LINKED_ADAPTER = 0xc01e0430,
    STATUS_GRAPHICS_LEADLINK_NOT_ENUMERATED = 0xc01e0431,
    STATUS_GRAPHICS_CHAINLINKS_NOT_ENUMERATED = 0xc01e0432,
    STATUS_GRAPHICS_ADAPTER_CHAIN_NOT_READY = 0xc01e0433,
    STATUS_GRAPHICS_CHAINLINKS_NOT_STARTED = 0xc01e0434,
    STATUS_GRAPHICS_CHAINLINKS_NOT_POWERED_ON = 0xc01e0435,
    STATUS_GRAPHICS_INCONSISTENT_DEVICE_LINK_STATE = 0xc01e0436,
    STATUS_GRAPHICS_NOT_POST_DEVICE_DRIVER = 0xc01e0438,
    STATUS_GRAPHICS_ADAPTER_ACCESS_NOT_EXCLUDED = 0xc01e043b,
    STATUS_GRAPHICS_OPM_NOT_SUPPORTED = 0xc01e0500,
    STATUS_GRAPHICS_COPP_NOT_SUPPORTED = 0xc01e0501,
    STATUS_GRAPHICS_UAB_NOT_SUPPORTED = 0xc01e0502,
    STATUS_GRAPHICS_OPM_INVALID_ENCRYPTED_PARAMETERS = 0xc01e0503,
    STATUS_GRAPHICS_OPM_NO_PROTECTED_OUTPUTS_EXIST = 0xc01e0505,
    STATUS_GRAPHICS_OPM_INTERNAL_ERROR = 0xc01e050b,
    STATUS_GRAPHICS_OPM_INVALID_HANDLE = 0xc01e050c,
    STATUS_GRAPHICS_PVP_INVALID_CERTIFICATE_LENGTH = 0xc01e050e,
    STATUS_GRAPHICS_OPM_SPANNING_MODE_ENABLED = 0xc01e050f,
    STATUS_GRAPHICS_OPM_THEATER_MODE_ENABLED = 0xc01e0510,
    STATUS_GRAPHICS_PVP_HFS_FAILED = 0xc01e0511,
    STATUS_GRAPHICS_OPM_INVALID_SRM = 0xc01e0512,
    STATUS_GRAPHICS_OPM_OUTPUT_DOES_NOT_SUPPORT_HDCP = 0xc01e0513,
    STATUS_GRAPHICS_OPM_OUTPUT_DOES_NOT_SUPPORT_ACP = 0xc01e0514,
    STATUS_GRAPHICS_OPM_OUTPUT_DOES_NOT_SUPPORT_CGMSA = 0xc01e0515,
    STATUS_GRAPHICS_OPM_HDCP_SRM_NEVER_SET = 0xc01e0516,
    STATUS_GRAPHICS_OPM_RESOLUTION_TOO_HIGH = 0xc01e0517,
    STATUS_GRAPHICS_OPM_ALL_HDCP_HARDWARE_ALREADY_IN_USE = 0xc01e0518,
    STATUS_GRAPHICS_OPM_PROTECTED_OUTPUT_NO_LONGER_EXISTS = 0xc01e051a,
    STATUS_GRAPHICS_OPM_PROTECTED_OUTPUT_DOES_NOT_HAVE_COPP_SEMANTICS = 0xc01e051c,
    STATUS_GRAPHICS_OPM_INVALID_INFORMATION_REQUEST = 0xc01e051d,
    STATUS_GRAPHICS_OPM_DRIVER_INTERNAL_ERROR = 0xc01e051e,
    STATUS_GRAPHICS_OPM_PROTECTED_OUTPUT_DOES_NOT_HAVE_OPM_SEMANTICS = 0xc01e051f,
    STATUS_GRAPHICS_OPM_SIGNALING_NOT_SUPPORTED = 0xc01e0520,
    STATUS_GRAPHICS_OPM_INVALID_CONFIGURATION_REQUEST = 0xc01e0521,
    STATUS_GRAPHICS_I2C_NOT_SUPPORTED = 0xc01e0580,
    STATUS_GRAPHICS_I2C_DEVICE_DOES_NOT_EXIST = 0xc01e0581,
    STATUS_GRAPHICS_I2C_ERROR_TRANSMITTING_DATA = 0xc01e0582,
    STATUS_GRAPHICS_I2C_ERROR_RECEIVING_DATA = 0xc01e0583,
    STATUS_GRAPHICS_DDCCI_VCP_NOT_SUPPORTED = 0xc01e0584,
    STATUS_GRAPHICS_DDCCI_INVALID_DATA = 0xc01e0585,
    STATUS_GRAPHICS_DDCCI_MONITOR_RETURNED_INVALID_TIMING_STATUS_BYTE = 0xc01e0586,
    STATUS_GRAPHICS_DDCCI_INVALID_CAPABILITIES_STRING = 0xc01e0587,
    STATUS_GRAPHICS_MCA_INTERNAL_ERROR = 0xc01e0588,
    STATUS_GRAPHICS_DDCCI_INVALID_MESSAGE_COMMAND = 0xc01e0589,
    STATUS_GRAPHICS_DDCCI_INVALID_MESSAGE_LENGTH = 0xc01e058a,
    STATUS_GRAPHICS_DDCCI_INVALID_MESSAGE_CHECKSUM = 0xc01e058b,
    STATUS_GRAPHICS_INVALID_PHYSICAL_MONITOR_HANDLE = 0xc01e058c,
    STATUS_GRAPHICS_MONITOR_NO_LONGER_EXISTS = 0xc01e058d,
    STATUS_GRAPHICS_ONLY_CONSOLE_SESSION_SUPPORTED = 0xc01e05e0,
    STATUS_GRAPHICS_NO_DISPLAY_DEVICE_CORRESPONDS_TO_NAME = 0xc01e05e1,
    STATUS_GRAPHICS_DISPLAY_DEVICE_NOT_ATTACHED_TO_DESKTOP = 0xc01e05e2,
    STATUS_GRAPHICS_MIRRORING_DEVICES_NOT_SUPPORTED = 0xc01e05e3,
    STATUS_GRAPHICS_INVALID_POINTER = 0xc01e05e4,
    STATUS_GRAPHICS_NO_MONITORS_CORRESPOND_TO_DISPLAY_DEVICE = 0xc01e05e5,
    STATUS_GRAPHICS_PARAMETER_ARRAY_TOO_SMALL = 0xc01e05e6,
    STATUS_GRAPHICS_INTERNAL_ERROR = 0xc01e05e7,
    STATUS_GRAPHICS_SESSION_TYPE_CHANGE_IN_PROGRESS = 0xc01e05e8,
    STATUS_FVE_LOCKED_VOLUME = 0xc0210000,
    STATUS_FVE_NOT_ENCRYPTED = 0xc0210001,
    STATUS_FVE_BAD_INFORMATION = 0xc0210002,
    STATUS_FVE_TOO_SMALL = 0xc0210003,
    STATUS_FVE_FAILED_WRONG_FS = 0xc0210004,
    STATUS_FVE_BAD_PARTITION_SIZE = 0xc0210005,
    STATUS_FVE_FS_NOT_EXTENDED = 0xc0210006,
    STATUS_FVE_FS_MOUNTED = 0xc0210007,
    STATUS_FVE_NO_LICENSE = 0xc0210008,
    STATUS_FVE_ACTION_NOT_ALLOWED = 0xc0210009,
    STATUS_FVE_BAD_DATA = 0xc021000a,
    STATUS_FVE_VOLUME_NOT_BOUND = 0xc021000b,
    STATUS_FVE_NOT_DATA_VOLUME = 0xc021000c,
    STATUS_FVE_CONV_READ_ERROR = 0xc021000d,
    STATUS_FVE_CONV_WRITE_ERROR = 0xc021000e,
    STATUS_FVE_OVERLAPPED_UPDATE = 0xc021000f,
    STATUS_FVE_FAILED_SECTOR_SIZE = 0xc0210010,
    STATUS_FVE_FAILED_AUTHENTICATION = 0xc0210011,
    STATUS_FVE_NOT_OS_VOLUME = 0xc0210012,
    STATUS_FVE_KEYFILE_NOT_FOUND = 0xc0210013,
    STATUS_FVE_KEYFILE_INVALID = 0xc0210014,
    STATUS_FVE_KEYFILE_NO_VMK = 0xc0210015,
    STATUS_FVE_TPM_DISABLED = 0xc0210016,
    STATUS_FVE_TPM_SRK_AUTH_NOT_ZERO = 0xc0210017,
    STATUS_FVE_TPM_INVALID_PCR = 0xc0210018,
    STATUS_FVE_TPM_NO_VMK = 0xc0210019,
    STATUS_FVE_PIN_INVALID = 0xc021001a,
    STATUS_FVE_AUTH_INVALID_APPLICATION = 0xc021001b,
    STATUS_FVE_AUTH_INVALID_CONFIG = 0xc021001c,
    STATUS_FVE_DEBUGGER_ENABLED = 0xc021001d,
    STATUS_FVE_DRY_RUN_FAILED = 0xc021001e,
    STATUS_FVE_BAD_METADATA_POINTER = 0xc021001f,
    STATUS_FVE_OLD_METADATA_COPY = 0xc0210020,
    STATUS_FVE_REBOOT_REQUIRED = 0xc0210021,
    STATUS_FVE_RAW_ACCESS = 0xc0210022,
    STATUS_FVE_RAW_BLOCKED = 0xc0210023,
    STATUS_FVE_NO_AUTOUNLOCK_MASTER_KEY = 0xc0210024,
    STATUS_FVE_MOR_FAILED = 0xc0210025,
    STATUS_FVE_NO_FEATURE_LICENSE = 0xc0210026,
    STATUS_FVE_POLICY_USER_DISABLE_RDV_NOT_ALLOWED = 0xc0210027,
    STATUS_FVE_CONV_RECOVERY_FAILED = 0xc0210028,
    STATUS_FVE_VIRTUALIZED_SPACE_TOO_BIG = 0xc0210029,
    STATUS_FVE_INVALID_DATUM_TYPE = 0xc021002a,
    STATUS_FVE_VOLUME_TOO_SMALL = 0xc0210030,
    STATUS_FVE_ENH_PIN_INVALID = 0xc0210031,
    STATUS_FVE_FULL_ENCRYPTION_NOT_ALLOWED_ON_TP_STORAGE = 0xc0210032,
    STATUS_FVE_WIPE_NOT_ALLOWED_ON_TP_STORAGE = 0xc0210033,
    STATUS_FVE_NOT_ALLOWED_ON_CSV_STACK = 0xc0210034,
    STATUS_FVE_NOT_ALLOWED_ON_CLUSTER = 0xc0210035,
    STATUS_FVE_NOT_ALLOWED_TO_UPGRADE_WHILE_CONVERTING = 0xc0210036,
    STATUS_FVE_WIPE_CANCEL_NOT_APPLICABLE = 0xc0210037,
    STATUS_FVE_EDRIVE_DRY_RUN_FAILED = 0xc0210038,
    STATUS_FVE_SECUREBOOT_DISABLED = 0xc0210039,
    STATUS_FVE_SECUREBOOT_CONFIG_CHANGE = 0xc021003a,
    STATUS_FVE_DEVICE_LOCKEDOUT = 0xc021003b,
    STATUS_FVE_VOLUME_EXTEND_PREVENTS_EOW_DECRYPT = 0xc021003c,
    STATUS_FVE_NOT_DE_VOLUME = 0xc021003d,
    STATUS_FVE_PROTECTION_DISABLED = 0xc021003e,
    STATUS_FVE_PROTECTION_CANNOT_BE_DISABLED = 0xc021003f,
    STATUS_FVE_OSV_KSR_NOT_ALLOWED = 0xc0210040,
    STATUS_FWP_CALLOUT_NOT_FOUND = 0xc0220001,
    STATUS_FWP_CONDITION_NOT_FOUND = 0xc0220002,
    STATUS_FWP_FILTER_NOT_FOUND = 0xc0220003,
    STATUS_FWP_LAYER_NOT_FOUND = 0xc0220004,
    STATUS_FWP_PROVIDER_NOT_FOUND = 0xc0220005,
    STATUS_FWP_PROVIDER_CONTEXT_NOT_FOUND = 0xc0220006,
    STATUS_FWP_SUBLAYER_NOT_FOUND = 0xc0220007,
    STATUS_FWP_NOT_FOUND = 0xc0220008,
    STATUS_FWP_ALREADY_EXISTS = 0xc0220009,
    STATUS_FWP_IN_USE = 0xc022000a,
    STATUS_FWP_DYNAMIC_SESSION_IN_PROGRESS = 0xc022000b,
    STATUS_FWP_WRONG_SESSION = 0xc022000c,
    STATUS_FWP_NO_TXN_IN_PROGRESS = 0xc022000d,
    STATUS_FWP_TXN_IN_PROGRESS = 0xc022000e,
    STATUS_FWP_TXN_ABORTED = 0xc022000f,
    STATUS_FWP_SESSION_ABORTED = 0xc0220010,
    STATUS_FWP_INCOMPATIBLE_TXN = 0xc0220011,
    STATUS_FWP_TIMEOUT = 0xc0220012,
    STATUS_FWP_NET_EVENTS_DISABLED = 0xc0220013,
    STATUS_FWP_INCOMPATIBLE_LAYER = 0xc0220014,
    STATUS_FWP_KM_CLIENTS_ONLY = 0xc0220015,
    STATUS_FWP_LIFETIME_MISMATCH = 0xc0220016,
    STATUS_FWP_BUILTIN_OBJECT = 0xc0220017,
    STATUS_FWP_TOO_MANY_CALLOUTS = 0xc0220018,
    STATUS_FWP_NOTIFICATION_DROPPED = 0xc0220019,
    STATUS_FWP_TRAFFIC_MISMATCH = 0xc022001a,
    STATUS_FWP_INCOMPATIBLE_SA_STATE = 0xc022001b,
    STATUS_FWP_NULL_POINTER = 0xc022001c,
    STATUS_FWP_INVALID_ENUMERATOR = 0xc022001d,
    STATUS_FWP_INVALID_FLAGS = 0xc022001e,
    STATUS_FWP_INVALID_NET_MASK = 0xc022001f,
    STATUS_FWP_INVALID_RANGE = 0xc0220020,
    STATUS_FWP_INVALID_INTERVAL = 0xc0220021,
    STATUS_FWP_ZERO_LENGTH_ARRAY = 0xc0220022,
    STATUS_FWP_NULL_DISPLAY_NAME = 0xc0220023,
    STATUS_FWP_INVALID_ACTION_TYPE = 0xc0220024,
    STATUS_FWP_INVALID_WEIGHT = 0xc0220025,
    STATUS_FWP_MATCH_TYPE_MISMATCH = 0xc0220026,
    STATUS_FWP_TYPE_MISMATCH = 0xc0220027,
    STATUS_FWP_OUT_OF_BOUNDS = 0xc0220028,
    STATUS_FWP_RESERVED = 0xc0220029,
    STATUS_FWP_DUPLICATE_CONDITION = 0xc022002a,
    STATUS_FWP_DUPLICATE_KEYMOD = 0xc022002b,
    STATUS_FWP_ACTION_INCOMPATIBLE_WITH_LAYER = 0xc022002c,
    STATUS_FWP_ACTION_INCOMPATIBLE_WITH_SUBLAYER = 0xc022002d,
    STATUS_FWP_CONTEXT_INCOMPATIBLE_WITH_LAYER = 0xc022002e,
    STATUS_FWP_CONTEXT_INCOMPATIBLE_WITH_CALLOUT = 0xc022002f,
    STATUS_FWP_INCOMPATIBLE_AUTH_METHOD = 0xc0220030,
    STATUS_FWP_INCOMPATIBLE_DH_GROUP = 0xc0220031,
    STATUS_FWP_EM_NOT_SUPPORTED = 0xc0220032,
    STATUS_FWP_NEVER_MATCH = 0xc0220033,
    STATUS_FWP_PROVIDER_CONTEXT_MISMATCH = 0xc0220034,
    STATUS_FWP_INVALID_PARAMETER = 0xc0220035,
    STATUS_FWP_TOO_MANY_SUBLAYERS = 0xc0220036,
    STATUS_FWP_CALLOUT_NOTIFICATION_FAILED = 0xc0220037,
    STATUS_FWP_INVALID_AUTH_TRANSFORM = 0xc0220038,
    STATUS_FWP_INVALID_CIPHER_TRANSFORM = 0xc0220039,
    STATUS_FWP_INCOMPATIBLE_CIPHER_TRANSFORM = 0xc022003a,
    STATUS_FWP_INVALID_TRANSFORM_COMBINATION = 0xc022003b,
    STATUS_FWP_DUPLICATE_AUTH_METHOD = 0xc022003c,
    STATUS_FWP_INVALID_TUNNEL_ENDPOINT = 0xc022003d,
    STATUS_FWP_L2_DRIVER_NOT_READY = 0xc022003e,
    STATUS_FWP_KEY_DICTATOR_ALREADY_REGISTERED = 0xc022003f,
    STATUS_FWP_KEY_DICTATION_INVALID_KEYING_MATERIAL = 0xc0220040,
    STATUS_FWP_CONNECTIONS_DISABLED = 0xc0220041,
    STATUS_FWP_INVALID_DNS_NAME = 0xc0220042,
    STATUS_FWP_STILL_ON = 0xc0220043,
    STATUS_FWP_IKEEXT_NOT_RUNNING = 0xc0220044,
    STATUS_FWP_TCPIP_NOT_READY = 0xc0220100,
    STATUS_FWP_INJECT_HANDLE_CLOSING = 0xc0220101,
    STATUS_FWP_INJECT_HANDLE_STALE = 0xc0220102,
    STATUS_FWP_CANNOT_PEND = 0xc0220103,
    STATUS_FWP_DROP_NOICMP = 0xc0220104,
    STATUS_NDIS_CLOSING = 0xc0230002,
    STATUS_NDIS_BAD_VERSION = 0xc0230004,
    STATUS_NDIS_BAD_CHARACTERISTICS = 0xc0230005,
    STATUS_NDIS_ADAPTER_NOT_FOUND = 0xc0230006,
    STATUS_NDIS_OPEN_FAILED = 0xc0230007,
    STATUS_NDIS_DEVICE_FAILED = 0xc0230008,
    STATUS_NDIS_MULTICAST_FULL = 0xc0230009,
    STATUS_NDIS_MULTICAST_EXISTS = 0xc023000a,
    STATUS_NDIS_MULTICAST_NOT_FOUND = 0xc023000b,
    STATUS_NDIS_REQUEST_ABORTED = 0xc023000c,
    STATUS_NDIS_RESET_IN_PROGRESS = 0xc023000d,
    STATUS_NDIS_INVALID_PACKET = 0xc023000f,
    STATUS_NDIS_INVALID_DEVICE_REQUEST = 0xc0230010,
    STATUS_NDIS_ADAPTER_NOT_READY = 0xc0230011,
    STATUS_NDIS_INVALID_LENGTH = 0xc0230014,
    STATUS_NDIS_INVALID_DATA = 0xc0230015,
    STATUS_NDIS_BUFFER_TOO_SHORT = 0xc0230016,
    STATUS_NDIS_INVALID_OID = 0xc0230017,
    STATUS_NDIS_ADAPTER_REMOVED = 0xc0230018,
    STATUS_NDIS_UNSUPPORTED_MEDIA = 0xc0230019,
    STATUS_NDIS_GROUP_ADDRESS_IN_USE = 0xc023001a,
    STATUS_NDIS_FILE_NOT_FOUND = 0xc023001b,
    STATUS_NDIS_ERROR_READING_FILE = 0xc023001c,
    STATUS_NDIS_ALREADY_MAPPED = 0xc023001d,
    STATUS_NDIS_RESOURCE_CONFLICT = 0xc023001e,
    STATUS_NDIS_MEDIA_DISCONNECTED = 0xc023001f,
    STATUS_NDIS_INVALID_ADDRESS = 0xc0230022,
    STATUS_NDIS_PAUSED = 0xc023002a,
    STATUS_NDIS_INTERFACE_NOT_FOUND = 0xc023002b,
    STATUS_NDIS_UNSUPPORTED_REVISION = 0xc023002c,
    STATUS_NDIS_INVALID_PORT = 0xc023002d,
    STATUS_NDIS_INVALID_PORT_STATE = 0xc023002e,
    STATUS_NDIS_LOW_POWER_STATE = 0xc023002f,
    STATUS_NDIS_REINIT_REQUIRED = 0xc0230030,
    STATUS_NDIS_NO_QUEUES = 0xc0230031,
    STATUS_NDIS_NOT_SUPPORTED = 0xc02300bb,
    STATUS_NDIS_OFFLOAD_POLICY = 0xc023100f,
    STATUS_NDIS_OFFLOAD_CONNECTION_REJECTED = 0xc0231012,
    STATUS_NDIS_OFFLOAD_PATH_REJECTED = 0xc0231013,
    STATUS_NDIS_DOT11_AUTO_CONFIG_ENABLED = 0xc0232000,
    STATUS_NDIS_DOT11_MEDIA_IN_USE = 0xc0232001,
    STATUS_NDIS_DOT11_POWER_STATE_INVALID = 0xc0232002,
    STATUS_NDIS_PM_WOL_PATTERN_LIST_FULL = 0xc0232003,
    STATUS_NDIS_PM_PROTOCOL_OFFLOAD_LIST_FULL = 0xc0232004,
    STATUS_NDIS_DOT11_AP_CHANNEL_CURRENTLY_NOT_AVAILABLE = 0xc0232005,
    STATUS_NDIS_DOT11_AP_BAND_CURRENTLY_NOT_AVAILABLE = 0xc0232006,
    STATUS_NDIS_DOT11_AP_CHANNEL_NOT_ALLOWED = 0xc0232007,
    STATUS_NDIS_DOT11_AP_BAND_NOT_ALLOWED = 0xc0232008,
    STATUS_QUIC_HANDSHAKE_FAILURE = 0xc0240000,
    STATUS_QUIC_VER_NEG_FAILURE = 0xc0240001,
    STATUS_TPM_ERROR_MASK = 0xc0290000,
    STATUS_TPM_AUTHFAIL = 0xc0290001,
    STATUS_TPM_BADINDEX = 0xc0290002,
    STATUS_TPM_BAD_PARAMETER = 0xc0290003,
    STATUS_TPM_AUDITFAILURE = 0xc0290004,
    STATUS_TPM_CLEAR_DISABLED = 0xc0290005,
    STATUS_TPM_DEACTIVATED = 0xc0290006,
    STATUS_TPM_DISABLED = 0xc0290007,
    STATUS_TPM_DISABLED_CMD = 0xc0290008,
    STATUS_TPM_FAIL = 0xc0290009,
    STATUS_TPM_BAD_ORDINAL = 0xc029000a,
    STATUS_TPM_INSTALL_DISABLED = 0xc029000b,
    STATUS_TPM_INVALID_KEYHANDLE = 0xc029000c,
    STATUS_TPM_KEYNOTFOUND = 0xc029000d,
    STATUS_TPM_INAPPROPRIATE_ENC = 0xc029000e,
    STATUS_TPM_MIGRATEFAIL = 0xc029000f,
    STATUS_TPM_INVALID_PCR_INFO = 0xc0290010,
    STATUS_TPM_NOSPACE = 0xc0290011,
    STATUS_TPM_NOSRK = 0xc0290012,
    STATUS_TPM_NOTSEALED_BLOB = 0xc0290013,
    STATUS_TPM_OWNER_SET = 0xc0290014,
    STATUS_TPM_RESOURCES = 0xc0290015,
    STATUS_TPM_SHORTRANDOM = 0xc0290016,
    STATUS_TPM_SIZE = 0xc0290017,
    STATUS_TPM_WRONGPCRVAL = 0xc0290018,
    STATUS_TPM_BAD_PARAM_SIZE = 0xc0290019,
    STATUS_TPM_SHA_THREAD = 0xc029001a,
    STATUS_TPM_SHA_ERROR = 0xc029001b,
    STATUS_TPM_FAILEDSELFTEST = 0xc029001c,
    STATUS_TPM_AUTH2FAIL = 0xc029001d,
    STATUS_TPM_BADTAG = 0xc029001e,
    STATUS_TPM_IOERROR = 0xc029001f,
    STATUS_TPM_ENCRYPT_ERROR = 0xc0290020,
    STATUS_TPM_DECRYPT_ERROR = 0xc0290021,
    STATUS_TPM_INVALID_AUTHHANDLE = 0xc0290022,
    STATUS_TPM_NO_ENDORSEMENT = 0xc0290023,
    STATUS_TPM_INVALID_KEYUSAGE = 0xc0290024,
    STATUS_TPM_WRONG_ENTITYTYPE = 0xc0290025,
    STATUS_TPM_INVALID_POSTINIT = 0xc0290026,
    STATUS_TPM_INAPPROPRIATE_SIG = 0xc0290027,
    STATUS_TPM_BAD_KEY_PROPERTY = 0xc0290028,
    STATUS_TPM_BAD_MIGRATION = 0xc0290029,
    STATUS_TPM_BAD_SCHEME = 0xc029002a,
    STATUS_TPM_BAD_DATASIZE = 0xc029002b,
    STATUS_TPM_BAD_MODE = 0xc029002c,
    STATUS_TPM_BAD_PRESENCE = 0xc029002d,
    STATUS_TPM_BAD_VERSION = 0xc029002e,
    STATUS_TPM_NO_WRAP_TRANSPORT = 0xc029002f,
    STATUS_TPM_AUDITFAIL_UNSUCCESSFUL = 0xc0290030,
    STATUS_TPM_AUDITFAIL_SUCCESSFUL = 0xc0290031,
    STATUS_TPM_NOTRESETABLE = 0xc0290032,
    STATUS_TPM_NOTLOCAL = 0xc0290033,
    STATUS_TPM_BAD_TYPE = 0xc0290034,
    STATUS_TPM_INVALID_RESOURCE = 0xc0290035,
    STATUS_TPM_NOTFIPS = 0xc0290036,
    STATUS_TPM_INVALID_FAMILY = 0xc0290037,
    STATUS_TPM_NO_NV_PERMISSION = 0xc0290038,
    STATUS_TPM_REQUIRES_SIGN = 0xc0290039,
    STATUS_TPM_KEY_NOTSUPPORTED = 0xc029003a,
    STATUS_TPM_AUTH_CONFLICT = 0xc029003b,
    STATUS_TPM_AREA_LOCKED = 0xc029003c,
    STATUS_TPM_BAD_LOCALITY = 0xc029003d,
    STATUS_TPM_READ_ONLY = 0xc029003e,
    STATUS_TPM_PER_NOWRITE = 0xc029003f,
    STATUS_TPM_FAMILYCOUNT = 0xc0290040,
    STATUS_TPM_WRITE_LOCKED = 0xc0290041,
    STATUS_TPM_BAD_ATTRIBUTES = 0xc0290042,
    STATUS_TPM_INVALID_STRUCTURE = 0xc0290043,
    STATUS_TPM_KEY_OWNER_CONTROL = 0xc0290044,
    STATUS_TPM_BAD_COUNTER = 0xc0290045,
    STATUS_TPM_NOT_FULLWRITE = 0xc0290046,
    STATUS_TPM_CONTEXT_GAP = 0xc0290047,
    STATUS_TPM_MAXNVWRITES = 0xc0290048,
    STATUS_TPM_NOOPERATOR = 0xc0290049,
    STATUS_TPM_RESOURCEMISSING = 0xc029004a,
    STATUS_TPM_DELEGATE_LOCK = 0xc029004b,
    STATUS_TPM_DELEGATE_FAMILY = 0xc029004c,
    STATUS_TPM_DELEGATE_ADMIN = 0xc029004d,
    STATUS_TPM_TRANSPORT_NOTEXCLUSIVE = 0xc029004e,
    STATUS_TPM_OWNER_CONTROL = 0xc029004f,
    STATUS_TPM_DAA_RESOURCES = 0xc0290050,
    STATUS_TPM_DAA_INPUT_DATA0 = 0xc0290051,
    STATUS_TPM_DAA_INPUT_DATA1 = 0xc0290052,
    STATUS_TPM_DAA_ISSUER_SETTINGS = 0xc0290053,
    STATUS_TPM_DAA_TPM_SETTINGS = 0xc0290054,
    STATUS_TPM_DAA_STAGE = 0xc0290055,
    STATUS_TPM_DAA_ISSUER_VALIDITY = 0xc0290056,
    STATUS_TPM_DAA_WRONG_W = 0xc0290057,
    STATUS_TPM_BAD_HANDLE = 0xc0290058,
    STATUS_TPM_BAD_DELEGATE = 0xc0290059,
    STATUS_TPM_BADCONTEXT = 0xc029005a,
    STATUS_TPM_TOOMANYCONTEXTS = 0xc029005b,
    STATUS_TPM_MA_TICKET_SIGNATURE = 0xc029005c,
    STATUS_TPM_MA_DESTINATION = 0xc029005d,
    STATUS_TPM_MA_SOURCE = 0xc029005e,
    STATUS_TPM_MA_AUTHORITY = 0xc029005f,
    STATUS_TPM_PERMANENTEK = 0xc0290061,
    STATUS_TPM_BAD_SIGNATURE = 0xc0290062,
    STATUS_TPM_NOCONTEXTSPACE = 0xc0290063,
    STATUS_TPM_20_E_ASYMMETRIC = 0xc0290081,
    STATUS_TPM_20_E_ATTRIBUTES = 0xc0290082,
    STATUS_TPM_20_E_HASH = 0xc0290083,
    STATUS_TPM_20_E_VALUE = 0xc0290084,
    STATUS_TPM_20_E_HIERARCHY = 0xc0290085,
    STATUS_TPM_20_E_KEY_SIZE = 0xc0290087,
    STATUS_TPM_20_E_MGF = 0xc0290088,
    STATUS_TPM_20_E_MODE = 0xc0290089,
    STATUS_TPM_20_E_TYPE = 0xc029008a,
    STATUS_TPM_20_E_HANDLE = 0xc029008b,
    STATUS_TPM_20_E_KDF = 0xc029008c,
    STATUS_TPM_20_E_RANGE = 0xc029008d,
    STATUS_TPM_20_E_AUTH_FAIL = 0xc029008e,
    STATUS_TPM_20_E_NONCE = 0xc029008f,
    STATUS_TPM_20_E_PP = 0xc0290090,
    STATUS_TPM_20_E_SCHEME = 0xc0290092,
    STATUS_TPM_20_E_SIZE = 0xc0290095,
    STATUS_TPM_20_E_SYMMETRIC = 0xc0290096,
    STATUS_TPM_20_E_TAG = 0xc0290097,
    STATUS_TPM_20_E_SELECTOR = 0xc0290098,
    STATUS_TPM_20_E_INSUFFICIENT = 0xc029009a,
    STATUS_TPM_20_E_SIGNATURE = 0xc029009b,
    STATUS_TPM_20_E_KEY = 0xc029009c,
    STATUS_TPM_20_E_POLICY_FAIL = 0xc029009d,
    STATUS_TPM_20_E_INTEGRITY = 0xc029009f,
    STATUS_TPM_20_E_TICKET = 0xc02900a0,
    STATUS_TPM_20_E_RESERVED_BITS = 0xc02900a1,
    STATUS_TPM_20_E_BAD_AUTH = 0xc02900a2,
    STATUS_TPM_20_E_EXPIRED = 0xc02900a3,
    STATUS_TPM_20_E_POLICY_CC = 0xc02900a4,
    STATUS_TPM_20_E_BINDING = 0xc02900a5,
    STATUS_TPM_20_E_CURVE = 0xc02900a6,
    STATUS_TPM_20_E_ECC_POINT = 0xc02900a7,
    STATUS_TPM_20_E_INITIALIZE = 0xc0290100,
    STATUS_TPM_20_E_FAILURE = 0xc0290101,
    STATUS_TPM_20_E_SEQUENCE = 0xc0290103,
    STATUS_TPM_20_E_PRIVATE = 0xc029010b,
    STATUS_TPM_20_E_HMAC = 0xc0290119,
    STATUS_TPM_20_E_DISABLED = 0xc0290120,
    STATUS_TPM_20_E_EXCLUSIVE = 0xc0290121,
    STATUS_TPM_20_E_ECC_CURVE = 0xc0290123,
    STATUS_TPM_20_E_AUTH_TYPE = 0xc0290124,
    STATUS_TPM_20_E_AUTH_MISSING = 0xc0290125,
    STATUS_TPM_20_E_POLICY = 0xc0290126,
    STATUS_TPM_20_E_PCR = 0xc0290127,
    STATUS_TPM_20_E_PCR_CHANGED = 0xc0290128,
    STATUS_TPM_20_E_UPGRADE = 0xc029012d,
    STATUS_TPM_20_E_TOO_MANY_CONTEXTS = 0xc029012e,
    STATUS_TPM_20_E_AUTH_UNAVAILABLE = 0xc029012f,
    STATUS_TPM_20_E_REBOOT = 0xc0290130,
    STATUS_TPM_20_E_UNBALANCED = 0xc0290131,
    STATUS_TPM_20_E_COMMAND_SIZE = 0xc0290142,
    STATUS_TPM_20_E_COMMAND_CODE = 0xc0290143,
    STATUS_TPM_20_E_AUTHSIZE = 0xc0290144,
    STATUS_TPM_20_E_AUTH_CONTEXT = 0xc0290145,
    STATUS_TPM_20_E_NV_RANGE = 0xc0290146,
    STATUS_TPM_20_E_NV_SIZE = 0xc0290147,
    STATUS_TPM_20_E_NV_LOCKED = 0xc0290148,
    STATUS_TPM_20_E_NV_AUTHORIZATION = 0xc0290149,
    STATUS_TPM_20_E_NV_UNINITIALIZED = 0xc029014a,
    STATUS_TPM_20_E_NV_SPACE = 0xc029014b,
    STATUS_TPM_20_E_NV_DEFINED = 0xc029014c,
    STATUS_TPM_20_E_BAD_CONTEXT = 0xc0290150,
    STATUS_TPM_20_E_CPHASH = 0xc0290151,
    STATUS_TPM_20_E_PARENT = 0xc0290152,
    STATUS_TPM_20_E_NEEDS_TEST = 0xc0290153,
    STATUS_TPM_20_E_NO_RESULT = 0xc0290154,
    STATUS_TPM_20_E_SENSITIVE = 0xc0290155,
    STATUS_TPM_COMMAND_BLOCKED = 0xc0290400,
    STATUS_TPM_INVALID_HANDLE = 0xc0290401,
    STATUS_TPM_DUPLICATE_VHANDLE = 0xc0290402,
    STATUS_TPM_EMBEDDED_COMMAND_BLOCKED = 0xc0290403,
    STATUS_TPM_EMBEDDED_COMMAND_UNSUPPORTED = 0xc0290404,
    STATUS_TPM_RETRY = 0xc0290800,
    STATUS_TPM_NEEDS_SELFTEST = 0xc0290801,
    STATUS_TPM_DOING_SELFTEST = 0xc0290802,
    STATUS_TPM_DEFEND_LOCK_RUNNING = 0xc0290803,
    STATUS_TPM_COMMAND_CANCELED = 0xc0291001,
    STATUS_TPM_TOO_MANY_CONTEXTS = 0xc0291002,
    STATUS_TPM_NOT_FOUND = 0xc0291003,
    STATUS_TPM_ACCESS_DENIED = 0xc0291004,
    STATUS_TPM_INSUFFICIENT_BUFFER = 0xc0291005,
    STATUS_TPM_PPI_FUNCTION_UNSUPPORTED = 0xc0291006,
    STATUS_PCP_ERROR_MASK = 0xc0292000,
    STATUS_PCP_DEVICE_NOT_READY = 0xc0292001,
    STATUS_PCP_INVALID_HANDLE = 0xc0292002,
    STATUS_PCP_INVALID_PARAMETER = 0xc0292003,
    STATUS_PCP_FLAG_NOT_SUPPORTED = 0xc0292004,
    STATUS_PCP_NOT_SUPPORTED = 0xc0292005,
    STATUS_PCP_BUFFER_TOO_SMALL = 0xc0292006,
    STATUS_PCP_INTERNAL_ERROR = 0xc0292007,
    STATUS_PCP_AUTHENTICATION_FAILED = 0xc0292008,
    STATUS_PCP_AUTHENTICATION_IGNORED = 0xc0292009,
    STATUS_PCP_POLICY_NOT_FOUND = 0xc029200a,
    STATUS_PCP_PROFILE_NOT_FOUND = 0xc029200b,
    STATUS_PCP_VALIDATION_FAILED = 0xc029200c,
    STATUS_PCP_DEVICE_NOT_FOUND = 0xc029200d,
    STATUS_PCP_WRONG_PARENT = 0xc029200e,
    STATUS_PCP_KEY_NOT_LOADED = 0xc029200f,
    STATUS_PCP_NO_KEY_CERTIFICATION = 0xc0292010,
    STATUS_PCP_KEY_NOT_FINALIZED = 0xc0292011,
    STATUS_PCP_ATTESTATION_CHALLENGE_NOT_SET = 0xc0292012,
    STATUS_PCP_NOT_PCR_BOUND = 0xc0292013,
    STATUS_PCP_KEY_ALREADY_FINALIZED = 0xc0292014,
    STATUS_PCP_KEY_USAGE_POLICY_NOT_SUPPORTED = 0xc0292015,
    STATUS_PCP_KEY_USAGE_POLICY_INVALID = 0xc0292016,
    STATUS_PCP_SOFT_KEY_ERROR = 0xc0292017,
    STATUS_PCP_KEY_NOT_AUTHENTICATED = 0xc0292018,
    STATUS_PCP_KEY_NOT_AIK = 0xc0292019,
    STATUS_PCP_KEY_NOT_SIGNING_KEY = 0xc029201a,
    STATUS_PCP_LOCKED_OUT = 0xc029201b,
    STATUS_PCP_CLAIM_TYPE_NOT_SUPPORTED = 0xc029201c,
    STATUS_PCP_TPM_VERSION_NOT_SUPPORTED = 0xc029201d,
    STATUS_PCP_BUFFER_LENGTH_MISMATCH = 0xc029201e,
    STATUS_PCP_IFX_RSA_KEY_CREATION_BLOCKED = 0xc029201f,
    STATUS_PCP_TICKET_MISSING = 0xc0292020,
    STATUS_PCP_RAW_POLICY_NOT_SUPPORTED = 0xc0292021,
    STATUS_PCP_KEY_HANDLE_INVALIDATED = 0xc0292022,
    STATUS_RTPM_NO_RESULT = 0xc0293002,
    STATUS_RTPM_PCR_READ_INCOMPLETE = 0xc0293003,
    STATUS_RTPM_INVALID_CONTEXT = 0xc0293004,
    STATUS_RTPM_UNSUPPORTED_CMD = 0xc0293005,
    STATUS_TPM_ZERO_EXHAUST_ENABLED = 0xc0294000,
    STATUS_HV_INVALID_HYPERCALL_CODE = 0xc0350002,
    STATUS_HV_INVALID_HYPERCALL_INPUT = 0xc0350003,
    STATUS_HV_INVALID_ALIGNMENT = 0xc0350004,
    STATUS_HV_INVALID_PARAMETER = 0xc0350005,
    STATUS_HV_ACCESS_DENIED = 0xc0350006,
    STATUS_HV_INVALID_PARTITION_STATE = 0xc0350007,
    STATUS_HV_OPERATION_DENIED = 0xc0350008,
    STATUS_HV_UNKNOWN_PROPERTY = 0xc0350009,
    STATUS_HV_PROPERTY_VALUE_OUT_OF_RANGE = 0xc035000a,
    STATUS_HV_INSUFFICIENT_MEMORY = 0xc035000b,
    STATUS_HV_PARTITION_TOO_DEEP = 0xc035000c,
    STATUS_HV_INVALID_PARTITION_ID = 0xc035000d,
    STATUS_HV_INVALID_VP_INDEX = 0xc035000e,
    STATUS_HV_INVALID_PORT_ID = 0xc0350011,
    STATUS_HV_INVALID_CONNECTION_ID = 0xc0350012,
    STATUS_HV_INSUFFICIENT_BUFFERS = 0xc0350013,
    STATUS_HV_NOT_ACKNOWLEDGED = 0xc0350014,
    STATUS_HV_INVALID_VP_STATE = 0xc0350015,
    STATUS_HV_ACKNOWLEDGED = 0xc0350016,
    STATUS_HV_INVALID_SAVE_RESTORE_STATE = 0xc0350017,
    STATUS_HV_INVALID_SYNIC_STATE = 0xc0350018,
    STATUS_HV_OBJECT_IN_USE = 0xc0350019,
    STATUS_HV_INVALID_PROXIMITY_DOMAIN_INFO = 0xc035001a,
    STATUS_HV_NO_DATA = 0xc035001b,
    STATUS_HV_INACTIVE = 0xc035001c,
    STATUS_HV_NO_RESOURCES = 0xc035001d,
    STATUS_HV_FEATURE_UNAVAILABLE = 0xc035001e,
    STATUS_HV_INSUFFICIENT_BUFFER = 0xc0350033,
    STATUS_HV_INSUFFICIENT_DEVICE_DOMAINS = 0xc0350038,
    STATUS_HV_CPUID_FEATURE_VALIDATION_ERROR = 0xc035003c,
    STATUS_HV_CPUID_XSAVE_FEATURE_VALIDATION_ERROR = 0xc035003d,
    STATUS_HV_PROCESSOR_STARTUP_TIMEOUT = 0xc035003e,
    STATUS_HV_SMX_ENABLED = 0xc035003f,
    STATUS_HV_INVALID_LP_INDEX = 0xc0350041,
    STATUS_HV_INVALID_REGISTER_VALUE = 0xc0350050,
    STATUS_HV_INVALID_VTL_STATE = 0xc0350051,
    STATUS_HV_NX_NOT_DETECTED = 0xc0350055,
    STATUS_HV_INVALID_DEVICE_ID = 0xc0350057,
    STATUS_HV_INVALID_DEVICE_STATE = 0xc0350058,
    STATUS_HV_PAGE_REQUEST_INVALID = 0xc0350060,
    STATUS_HV_INVALID_CPU_GROUP_ID = 0xc035006f,
    STATUS_HV_INVALID_CPU_GROUP_STATE = 0xc0350070,
    STATUS_HV_OPERATION_FAILED = 0xc0350071,
    STATUS_HV_NOT_ALLOWED_WITH_NESTED_VIRT_ACTIVE = 0xc0350072,
    STATUS_HV_INSUFFICIENT_ROOT_MEMORY = 0xc0350073,
    STATUS_HV_EVENT_BUFFER_ALREADY_FREED = 0xc0350074,
    STATUS_HV_INSUFFICIENT_CONTIGUOUS_MEMORY = 0xc0350075,
    STATUS_HV_NOT_PRESENT = 0xc0351000,
    STATUS_IPSEC_BAD_SPI = 0xc0360001,
    STATUS_IPSEC_SA_LIFETIME_EXPIRED = 0xc0360002,
    STATUS_IPSEC_WRONG_SA = 0xc0360003,
    STATUS_IPSEC_REPLAY_CHECK_FAILED = 0xc0360004,
    STATUS_IPSEC_INVALID_PACKET = 0xc0360005,
    STATUS_IPSEC_INTEGRITY_CHECK_FAILED = 0xc0360006,
    STATUS_IPSEC_CLEAR_TEXT_DROP = 0xc0360007,
    STATUS_IPSEC_AUTH_FIREWALL_DROP = 0xc0360008,
    STATUS_IPSEC_THROTTLE_DROP = 0xc0360009,
    STATUS_IPSEC_DOSP_BLOCK = 0xc0368000,
    STATUS_IPSEC_DOSP_RECEIVED_MULTICAST = 0xc0368001,
    STATUS_IPSEC_DOSP_INVALID_PACKET = 0xc0368002,
    STATUS_IPSEC_DOSP_STATE_LOOKUP_FAILED = 0xc0368003,
    STATUS_IPSEC_DOSP_MAX_ENTRIES = 0xc0368004,
    STATUS_IPSEC_DOSP_KEYMOD_NOT_ALLOWED = 0xc0368005,
    STATUS_IPSEC_DOSP_MAX_PER_IP_RATELIMIT_QUEUES = 0xc0368006,
    STATUS_VID_DUPLICATE_HANDLER = 0xc0370001,
    STATUS_VID_TOO_MANY_HANDLERS = 0xc0370002,
    STATUS_VID_QUEUE_FULL = 0xc0370003,
    STATUS_VID_HANDLER_NOT_PRESENT = 0xc0370004,
    STATUS_VID_INVALID_OBJECT_NAME = 0xc0370005,
    STATUS_VID_PARTITION_NAME_TOO_LONG = 0xc0370006,
    STATUS_VID_MESSAGE_QUEUE_NAME_TOO_LONG = 0xc0370007,
    STATUS_VID_PARTITION_ALREADY_EXISTS = 0xc0370008,
    STATUS_VID_PARTITION_DOES_NOT_EXIST = 0xc0370009,
    STATUS_VID_PARTITION_NAME_NOT_FOUND = 0xc037000a,
    STATUS_VID_MESSAGE_QUEUE_ALREADY_EXISTS = 0xc037000b,
    STATUS_VID_EXCEEDED_MBP_ENTRY_MAP_LIMIT = 0xc037000c,
    STATUS_VID_MB_STILL_REFERENCED = 0xc037000d,
    STATUS_VID_CHILD_GPA_PAGE_SET_CORRUPTED = 0xc037000e,
    STATUS_VID_INVALID_NUMA_SETTINGS = 0xc037000f,
    STATUS_VID_INVALID_NUMA_NODE_INDEX = 0xc0370010,
    STATUS_VID_NOTIFICATION_QUEUE_ALREADY_ASSOCIATED = 0xc0370011,
    STATUS_VID_INVALID_MEMORY_BLOCK_HANDLE = 0xc0370012,
    STATUS_VID_PAGE_RANGE_OVERFLOW = 0xc0370013,
    STATUS_VID_INVALID_MESSAGE_QUEUE_HANDLE = 0xc0370014,
    STATUS_VID_INVALID_GPA_RANGE_HANDLE = 0xc0370015,
    STATUS_VID_NO_MEMORY_BLOCK_NOTIFICATION_QUEUE = 0xc0370016,
    STATUS_VID_MEMORY_BLOCK_LOCK_COUNT_EXCEEDED = 0xc0370017,
    STATUS_VID_INVALID_PPM_HANDLE = 0xc0370018,
    STATUS_VID_MBPS_ARE_LOCKED = 0xc0370019,
    STATUS_VID_MESSAGE_QUEUE_CLOSED = 0xc037001a,
    STATUS_VID_VIRTUAL_PROCESSOR_LIMIT_EXCEEDED = 0xc037001b,
    STATUS_VID_STOP_PENDING = 0xc037001c,
    STATUS_VID_INVALID_PROCESSOR_STATE = 0xc037001d,
    STATUS_VID_EXCEEDED_KM_CONTEXT_COUNT_LIMIT = 0xc037001e,
    STATUS_VID_KM_INTERFACE_ALREADY_INITIALIZED = 0xc037001f,
    STATUS_VID_MB_PROPERTY_ALREADY_SET_RESET = 0xc0370020,
    STATUS_VID_MMIO_RANGE_DESTROYED = 0xc0370021,
    STATUS_VID_INVALID_CHILD_GPA_PAGE_SET = 0xc0370022,
    STATUS_VID_RESERVE_PAGE_SET_IS_BEING_USED = 0xc0370023,
    STATUS_VID_RESERVE_PAGE_SET_TOO_SMALL = 0xc0370024,
    STATUS_VID_MBP_ALREADY_LOCKED_USING_RESERVED_PAGE = 0xc0370025,
    STATUS_VID_MBP_COUNT_EXCEEDED_LIMIT = 0xc0370026,
    STATUS_VID_SAVED_STATE_CORRUPT = 0xc0370027,
    STATUS_VID_SAVED_STATE_UNRECOGNIZED_ITEM = 0xc0370028,
    STATUS_VID_SAVED_STATE_INCOMPATIBLE = 0xc0370029,
    STATUS_VID_VTL_ACCESS_DENIED = 0xc037002a,
    STATUS_VOLMGR_DATABASE_FULL = 0xc0380001,
    STATUS_VOLMGR_DISK_CONFIGURATION_CORRUPTED = 0xc0380002,
    STATUS_VOLMGR_DISK_CONFIGURATION_NOT_IN_SYNC = 0xc0380003,
    STATUS_VOLMGR_PACK_CONFIG_UPDATE_FAILED = 0xc0380004,
    STATUS_VOLMGR_DISK_CONTAINS_NON_SIMPLE_VOLUME = 0xc0380005,
    STATUS_VOLMGR_DISK_DUPLICATE = 0xc0380006,
    STATUS_VOLMGR_DISK_DYNAMIC = 0xc0380007,
    STATUS_VOLMGR_DISK_ID_INVALID = 0xc0380008,
    STATUS_VOLMGR_DISK_INVALID = 0xc0380009,
    STATUS_VOLMGR_DISK_LAST_VOTER = 0xc038000a,
    STATUS_VOLMGR_DISK_LAYOUT_INVALID = 0xc038000b,
    STATUS_VOLMGR_DISK_LAYOUT_NON_BASIC_BETWEEN_BASIC_PARTITIONS = 0xc038000c,
    STATUS_VOLMGR_DISK_LAYOUT_NOT_CYLINDER_ALIGNED = 0xc038000d,
    STATUS_VOLMGR_DISK_LAYOUT_PARTITIONS_TOO_SMALL = 0xc038000e,
    STATUS_VOLMGR_DISK_LAYOUT_PRIMARY_BETWEEN_LOGICAL_PARTITIONS = 0xc038000f,
    STATUS_VOLMGR_DISK_LAYOUT_TOO_MANY_PARTITIONS = 0xc0380010,
    STATUS_VOLMGR_DISK_MISSING = 0xc0380011,
    STATUS_VOLMGR_DISK_NOT_EMPTY = 0xc0380012,
    STATUS_VOLMGR_DISK_NOT_ENOUGH_SPACE = 0xc0380013,
    STATUS_VOLMGR_DISK_REVECTORING_FAILED = 0xc0380014,
    STATUS_VOLMGR_DISK_SECTOR_SIZE_INVALID = 0xc0380015,
    STATUS_VOLMGR_DISK_SET_NOT_CONTAINED = 0xc0380016,
    STATUS_VOLMGR_DISK_USED_BY_MULTIPLE_MEMBERS = 0xc0380017,
    STATUS_VOLMGR_DISK_USED_BY_MULTIPLE_PLEXES = 0xc0380018,
    STATUS_VOLMGR_DYNAMIC_DISK_NOT_SUPPORTED = 0xc0380019,
    STATUS_VOLMGR_EXTENT_ALREADY_USED = 0xc038001a,
    STATUS_VOLMGR_EXTENT_NOT_CONTIGUOUS = 0xc038001b,
    STATUS_VOLMGR_EXTENT_NOT_IN_PUBLIC_REGION = 0xc038001c,
    STATUS_VOLMGR_EXTENT_NOT_SECTOR_ALIGNED = 0xc038001d,
    STATUS_VOLMGR_EXTENT_OVERLAPS_EBR_PARTITION = 0xc038001e,
    STATUS_VOLMGR_EXTENT_VOLUME_LENGTHS_DO_NOT_MATCH = 0xc038001f,
    STATUS_VOLMGR_FAULT_TOLERANT_NOT_SUPPORTED = 0xc0380020,
    STATUS_VOLMGR_INTERLEAVE_LENGTH_INVALID = 0xc0380021,
    STATUS_VOLMGR_MAXIMUM_REGISTERED_USERS = 0xc0380022,
    STATUS_VOLMGR_MEMBER_IN_SYNC = 0xc0380023,
    STATUS_VOLMGR_MEMBER_INDEX_DUPLICATE = 0xc0380024,
    STATUS_VOLMGR_MEMBER_INDEX_INVALID = 0xc0380025,
    STATUS_VOLMGR_MEMBER_MISSING = 0xc0380026,
    STATUS_VOLMGR_MEMBER_NOT_DETACHED = 0xc0380027,
    STATUS_VOLMGR_MEMBER_REGENERATING = 0xc0380028,
    STATUS_VOLMGR_ALL_DISKS_FAILED = 0xc0380029,
    STATUS_VOLMGR_NO_REGISTERED_USERS = 0xc038002a,
    STATUS_VOLMGR_NO_SUCH_USER = 0xc038002b,
    STATUS_VOLMGR_NOTIFICATION_RESET = 0xc038002c,
    STATUS_VOLMGR_NUMBER_OF_MEMBERS_INVALID = 0xc038002d,
    STATUS_VOLMGR_NUMBER_OF_PLEXES_INVALID = 0xc038002e,
    STATUS_VOLMGR_PACK_DUPLICATE = 0xc038002f,
    STATUS_VOLMGR_PACK_ID_INVALID = 0xc0380030,
    STATUS_VOLMGR_PACK_INVALID = 0xc0380031,
    STATUS_VOLMGR_PACK_NAME_INVALID = 0xc0380032,
    STATUS_VOLMGR_PACK_OFFLINE = 0xc0380033,
    STATUS_VOLMGR_PACK_HAS_QUORUM = 0xc0380034,
    STATUS_VOLMGR_PACK_WITHOUT_QUORUM = 0xc0380035,
    STATUS_VOLMGR_PARTITION_STYLE_INVALID = 0xc0380036,
    STATUS_VOLMGR_PARTITION_UPDATE_FAILED = 0xc0380037,
    STATUS_VOLMGR_PLEX_IN_SYNC = 0xc0380038,
    STATUS_VOLMGR_PLEX_INDEX_DUPLICATE = 0xc0380039,
    STATUS_VOLMGR_PLEX_INDEX_INVALID = 0xc038003a,
    STATUS_VOLMGR_PLEX_LAST_ACTIVE = 0xc038003b,
    STATUS_VOLMGR_PLEX_MISSING = 0xc038003c,
    STATUS_VOLMGR_PLEX_REGENERATING = 0xc038003d,
    STATUS_VOLMGR_PLEX_TYPE_INVALID = 0xc038003e,
    STATUS_VOLMGR_PLEX_NOT_RAID5 = 0xc038003f,
    STATUS_VOLMGR_PLEX_NOT_SIMPLE = 0xc0380040,
    STATUS_VOLMGR_STRUCTURE_SIZE_INVALID = 0xc0380041,
    STATUS_VOLMGR_TOO_MANY_NOTIFICATION_REQUESTS = 0xc0380042,
    STATUS_VOLMGR_TRANSACTION_IN_PROGRESS = 0xc0380043,
    STATUS_VOLMGR_UNEXPECTED_DISK_LAYOUT_CHANGE = 0xc0380044,
    STATUS_VOLMGR_VOLUME_CONTAINS_MISSING_DISK = 0xc0380045,
    STATUS_VOLMGR_VOLUME_ID_INVALID = 0xc0380046,
    STATUS_VOLMGR_VOLUME_LENGTH_INVALID = 0xc0380047,
    STATUS_VOLMGR_VOLUME_LENGTH_NOT_SECTOR_SIZE_MULTIPLE = 0xc0380048,
    STATUS_VOLMGR_VOLUME_NOT_MIRRORED = 0xc0380049,
    STATUS_VOLMGR_VOLUME_NOT_RETAINED = 0xc038004a,
    STATUS_VOLMGR_VOLUME_OFFLINE = 0xc038004b,
    STATUS_VOLMGR_VOLUME_RETAINED = 0xc038004c,
    STATUS_VOLMGR_NUMBER_OF_EXTENTS_INVALID = 0xc038004d,
    STATUS_VOLMGR_DIFFERENT_SECTOR_SIZE = 0xc038004e,
    STATUS_VOLMGR_BAD_BOOT_DISK = 0xc038004f,
    STATUS_VOLMGR_PACK_CONFIG_OFFLINE = 0xc0380050,
    STATUS_VOLMGR_PACK_CONFIG_ONLINE = 0xc0380051,
    STATUS_VOLMGR_NOT_PRIMARY_PACK = 0xc0380052,
    STATUS_VOLMGR_PACK_LOG_UPDATE_FAILED = 0xc0380053,
    STATUS_VOLMGR_NUMBER_OF_DISKS_IN_PLEX_INVALID = 0xc0380054,
    STATUS_VOLMGR_NUMBER_OF_DISKS_IN_MEMBER_INVALID = 0xc0380055,
    STATUS_VOLMGR_VOLUME_MIRRORED = 0xc0380056,
    STATUS_VOLMGR_PLEX_NOT_SIMPLE_SPANNED = 0xc0380057,
    STATUS_VOLMGR_NO_VALID_LOG_COPIES = 0xc0380058,
    STATUS_VOLMGR_PRIMARY_PACK_PRESENT = 0xc0380059,
    STATUS_VOLMGR_NUMBER_OF_DISKS_INVALID = 0xc038005a,
    STATUS_VOLMGR_MIRROR_NOT_SUPPORTED = 0xc038005b,
    STATUS_VOLMGR_RAID5_NOT_SUPPORTED = 0xc038005c,
    STATUS_BCD_TOO_MANY_ELEMENTS = 0xc0390002,
    STATUS_VHD_DRIVE_FOOTER_MISSING = 0xc03a0001,
    STATUS_VHD_DRIVE_FOOTER_CHECKSUM_MISMATCH = 0xc03a0002,
    STATUS_VHD_DRIVE_FOOTER_CORRUPT = 0xc03a0003,
    STATUS_VHD_FORMAT_UNKNOWN = 0xc03a0004,
    STATUS_VHD_FORMAT_UNSUPPORTED_VERSION = 0xc03a0005,
    STATUS_VHD_SPARSE_HEADER_CHECKSUM_MISMATCH = 0xc03a0006,
    STATUS_VHD_SPARSE_HEADER_UNSUPPORTED_VERSION = 0xc03a0007,
    STATUS_VHD_SPARSE_HEADER_CORRUPT = 0xc03a0008,
    STATUS_VHD_BLOCK_ALLOCATION_FAILURE = 0xc03a0009,
    STATUS_VHD_BLOCK_ALLOCATION_TABLE_CORRUPT = 0xc03a000a,
    STATUS_VHD_INVALID_BLOCK_SIZE = 0xc03a000b,
    STATUS_VHD_BITMAP_MISMATCH = 0xc03a000c,
    STATUS_VHD_PARENT_VHD_NOT_FOUND = 0xc03a000d,
    STATUS_VHD_CHILD_PARENT_ID_MISMATCH = 0xc03a000e,
    STATUS_VHD_CHILD_PARENT_TIMESTAMP_MISMATCH = 0xc03a000f,
    STATUS_VHD_METADATA_READ_FAILURE = 0xc03a0010,
    STATUS_VHD_METADATA_WRITE_FAILURE = 0xc03a0011,
    STATUS_VHD_INVALID_SIZE = 0xc03a0012,
    STATUS_VHD_INVALID_FILE_SIZE = 0xc03a0013,
    STATUS_VIRTDISK_PROVIDER_NOT_FOUND = 0xc03a0014,
    STATUS_VIRTDISK_NOT_VIRTUAL_DISK = 0xc03a0015,
    STATUS_VHD_PARENT_VHD_ACCESS_DENIED = 0xc03a0016,
    STATUS_VHD_CHILD_PARENT_SIZE_MISMATCH = 0xc03a0017,
    STATUS_VHD_DIFFERENCING_CHAIN_CYCLE_DETECTED = 0xc03a0018,
    STATUS_VHD_DIFFERENCING_CHAIN_ERROR_IN_PARENT = 0xc03a0019,
    STATUS_VIRTUAL_DISK_LIMITATION = 0xc03a001a,
    STATUS_VHD_INVALID_TYPE = 0xc03a001b,
    STATUS_VHD_INVALID_STATE = 0xc03a001c,
    STATUS_VIRTDISK_UNSUPPORTED_DISK_SECTOR_SIZE = 0xc03a001d,
    STATUS_VIRTDISK_DISK_ALREADY_OWNED = 0xc03a001e,
    STATUS_VIRTDISK_DISK_ONLINE_AND_WRITABLE = 0xc03a001f,
    STATUS_CTLOG_TRACKING_NOT_INITIALIZED = 0xc03a0020,
    STATUS_CTLOG_LOGFILE_SIZE_EXCEEDED_MAXSIZE = 0xc03a0021,
    STATUS_CTLOG_VHD_CHANGED_OFFLINE = 0xc03a0022,
    STATUS_CTLOG_INVALID_TRACKING_STATE = 0xc03a0023,
    STATUS_CTLOG_INCONSISTENT_TRACKING_FILE = 0xc03a0024,
    STATUS_VHD_METADATA_FULL = 0xc03a0028,
    STATUS_VHD_INVALID_CHANGE_TRACKING_ID = 0xc03a0029,
    STATUS_VHD_CHANGE_TRACKING_DISABLED = 0xc03a002a,
    STATUS_VHD_MISSING_CHANGE_TRACKING_INFORMATION = 0xc03a0030,
    STATUS_VHD_RESIZE_WOULD_TRUNCATE_DATA = 0xc03a0031,
    STATUS_VHD_COULD_NOT_COMPUTE_MINIMUM_VIRTUAL_SIZE = 0xc03a0032,
    STATUS_VHD_ALREADY_AT_OR_BELOW_MINIMUM_VIRTUAL_SIZE = 0xc03a0033,
    STATUS_RKF_KEY_NOT_FOUND = 0xc0400001,
    STATUS_RKF_DUPLICATE_KEY = 0xc0400002,
    STATUS_RKF_BLOB_FULL = 0xc0400003,
    STATUS_RKF_STORE_FULL = 0xc0400004,
    STATUS_RKF_FILE_BLOCKED = 0xc0400005,
    STATUS_RKF_ACTIVE_KEY = 0xc0400006,
    STATUS_RDBSS_RESTART_OPERATION = 0xc0410001,
    STATUS_RDBSS_CONTINUE_OPERATION = 0xc0410002,
    STATUS_RDBSS_POST_OPERATION = 0xc0410003,
    STATUS_RDBSS_RETRY_LOOKUP = 0xc0410004,
    STATUS_BTH_ATT_INVALID_HANDLE = 0xc0420001,
    STATUS_BTH_ATT_READ_NOT_PERMITTED = 0xc0420002,
    STATUS_BTH_ATT_WRITE_NOT_PERMITTED = 0xc0420003,
    STATUS_BTH_ATT_INVALID_PDU = 0xc0420004,
    STATUS_BTH_ATT_INSUFFICIENT_AUTHENTICATION = 0xc0420005,
    STATUS_BTH_ATT_REQUEST_NOT_SUPPORTED = 0xc0420006,
    STATUS_BTH_ATT_INVALID_OFFSET = 0xc0420007,
    STATUS_BTH_ATT_INSUFFICIENT_AUTHORIZATION = 0xc0420008,
    STATUS_BTH_ATT_PREPARE_QUEUE_FULL = 0xc0420009,
    STATUS_BTH_ATT_ATTRIBUTE_NOT_FOUND = 0xc042000a,
    STATUS_BTH_ATT_ATTRIBUTE_NOT_LONG = 0xc042000b,
    STATUS_BTH_ATT_INSUFFICIENT_ENCRYPTION_KEY_SIZE = 0xc042000c,
    STATUS_BTH_ATT_INVALID_ATTRIBUTE_VALUE_LENGTH = 0xc042000d,
    STATUS_BTH_ATT_UNLIKELY = 0xc042000e,
    STATUS_BTH_ATT_INSUFFICIENT_ENCRYPTION = 0xc042000f,
    STATUS_BTH_ATT_UNSUPPORTED_GROUP_TYPE = 0xc0420010,
    STATUS_BTH_ATT_INSUFFICIENT_RESOURCES = 0xc0420011,
    STATUS_BTH_ATT_UNKNOWN_ERROR = 0xc0421000,
    STATUS_SECUREBOOT_ROLLBACK_DETECTED = 0xc0430001,
    STATUS_SECUREBOOT_POLICY_VIOLATION = 0xc0430002,
    STATUS_SECUREBOOT_INVALID_POLICY = 0xc0430003,
    STATUS_SECUREBOOT_POLICY_PUBLISHER_NOT_FOUND = 0xc0430004,
    STATUS_SECUREBOOT_POLICY_NOT_SIGNED = 0xc0430005,
    STATUS_SECUREBOOT_FILE_REPLACED = 0xc0430007,
    STATUS_SECUREBOOT_POLICY_NOT_AUTHORIZED = 0xc0430008,
    STATUS_SECUREBOOT_POLICY_UNKNOWN = 0xc0430009,
    STATUS_SECUREBOOT_POLICY_MISSING_ANTIROLLBACKVERSION = 0xc043000a,
    STATUS_SECUREBOOT_PLATFORM_ID_MISMATCH = 0xc043000b,
    STATUS_SECUREBOOT_POLICY_ROLLBACK_DETECTED = 0xc043000c,
    STATUS_SECUREBOOT_POLICY_UPGRADE_MISMATCH = 0xc043000d,
    STATUS_SECUREBOOT_REQUIRED_POLICY_FILE_MISSING = 0xc043000e,
    STATUS_SECUREBOOT_NOT_BASE_POLICY = 0xc043000f,
    STATUS_SECUREBOOT_NOT_SUPPLEMENTAL_POLICY = 0xc0430010,
    STATUS_AUDIO_ENGINE_NODE_NOT_FOUND = 0xc0440001,
    STATUS_HDAUDIO_EMPTY_CONNECTION_LIST = 0xc0440002,
    STATUS_HDAUDIO_CONNECTION_LIST_NOT_SUPPORTED = 0xc0440003,
    STATUS_HDAUDIO_NO_LOGICAL_DEVICES_CREATED = 0xc0440004,
    STATUS_HDAUDIO_NULL_LINKED_LIST_ENTRY = 0xc0440005,
    STATUS_VSM_NOT_INITIALIZED = 0xc0450000,
    STATUS_VSM_DMA_PROTECTION_NOT_IN_USE = 0xc0450001,
    STATUS_VOLSNAP_BOOTFILE_NOT_VALID = 0xc0500003,
    STATUS_VOLSNAP_ACTIVATION_TIMEOUT = 0xc0500004,
    STATUS_IO_PREEMPTED = 0xc0510001,
    STATUS_SVHDX_ERROR_STORED = 0xc05c0000,
    STATUS_SVHDX_ERROR_NOT_AVAILABLE = 0xc05cff00,
    STATUS_SVHDX_UNIT_ATTENTION_AVAILABLE = 0xc05cff01,
    STATUS_SVHDX_UNIT_ATTENTION_CAPACITY_DATA_CHANGED = 0xc05cff02,
    STATUS_SVHDX_UNIT_ATTENTION_RESERVATIONS_PREEMPTED = 0xc05cff03,
    STATUS_SVHDX_UNIT_ATTENTION_RESERVATIONS_RELEASED = 0xc05cff04,
    STATUS_SVHDX_UNIT_ATTENTION_REGISTRATIONS_PREEMPTED = 0xc05cff05,
    STATUS_SVHDX_UNIT_ATTENTION_OPERATING_DEFINITION_CHANGED = 0xc05cff06,
    STATUS_SVHDX_RESERVATION_CONFLICT = 0xc05cff07,
    STATUS_SVHDX_WRONG_FILE_TYPE = 0xc05cff08,
    STATUS_SVHDX_VERSION_MISMATCH = 0xc05cff09,
    STATUS_VHD_SHARED = 0xc05cff0a,
    STATUS_SVHDX_NO_INITIATOR = 0xc05cff0b,
    STATUS_VHDSET_BACKING_STORAGE_NOT_FOUND = 0xc05cff0c,
    STATUS_SMB_NO_PREAUTH_INTEGRITY_HASH_OVERLAP = 0xc05d0000,
    STATUS_SMB_BAD_CLUSTER_DIALECT = 0xc05d0001,
    STATUS_SMB_GUEST_LOGON_BLOCKED = 0xc05d0002,
    STATUS_SPACES_FAULT_DOMAIN_TYPE_INVALID = 0xc0e70001,
    STATUS_SPACES_RESILIENCY_TYPE_INVALID = 0xc0e70003,
    STATUS_SPACES_DRIVE_SECTOR_SIZE_INVALID = 0xc0e70004,
    STATUS_SPACES_DRIVE_REDUNDANCY_INVALID = 0xc0e70006,
    STATUS_SPACES_NUMBER_OF_DATA_COPIES_INVALID = 0xc0e70007,
    STATUS_SPACES_INTERLEAVE_LENGTH_INVALID = 0xc0e70009,
    STATUS_SPACES_NUMBER_OF_COLUMNS_INVALID = 0xc0e7000a,
    STATUS_SPACES_NOT_ENOUGH_DRIVES = 0xc0e7000b,
    STATUS_SPACES_EXTENDED_ERROR = 0xc0e7000c,
    STATUS_SPACES_PROVISIONING_TYPE_INVALID = 0xc0e7000d,
    STATUS_SPACES_ALLOCATION_SIZE_INVALID = 0xc0e7000e,
    STATUS_SPACES_ENCLOSURE_AWARE_INVALID = 0xc0e7000f,
    STATUS_SPACES_WRITE_CACHE_SIZE_INVALID = 0xc0e70010,
    STATUS_SPACES_NUMBER_OF_GROUPS_INVALID = 0xc0e70011,
    STATUS_SPACES_DRIVE_OPERATIONAL_STATE_INVALID = 0xc0e70012,
    STATUS_SPACES_UPDATE_COLUMN_STATE = 0xc0e70013,
    STATUS_SPACES_MAP_REQUIRED = 0xc0e70014,
    STATUS_SPACES_UNSUPPORTED_VERSION = 0xc0e70015,
    STATUS_SPACES_CORRUPT_METADATA = 0xc0e70016,
    STATUS_SPACES_DRT_FULL = 0xc0e70017,
    STATUS_SPACES_INCONSISTENCY = 0xc0e70018,
    STATUS_SPACES_LOG_NOT_READY = 0xc0e70019,
    STATUS_SPACES_NO_REDUNDANCY = 0xc0e7001a,
    STATUS_SPACES_DRIVE_NOT_READY = 0xc0e7001b,
    STATUS_SPACES_DRIVE_SPLIT = 0xc0e7001c,
    STATUS_SPACES_DRIVE_LOST_DATA = 0xc0e7001d,
    STATUS_SPACES_ENTRY_INCOMPLETE = 0xc0e7001e,
    STATUS_SPACES_ENTRY_INVALID = 0xc0e7001f,
    STATUS_SPACES_MARK_DIRTY = 0xc0e70020,
    STATUS_SECCORE_INVALID_COMMAND = 0xc0e80000,
    STATUS_SYSTEM_INTEGRITY_ROLLBACK_DETECTED = 0xc0e90001,
    STATUS_SYSTEM_INTEGRITY_POLICY_VIOLATION = 0xc0e90002,
    STATUS_SYSTEM_INTEGRITY_INVALID_POLICY = 0xc0e90003,
    STATUS_SYSTEM_INTEGRITY_POLICY_NOT_SIGNED = 0xc0e90004,
    STATUS_SYSTEM_INTEGRITY_TOO_MANY_POLICIES = 0xc0e90005,
    STATUS_SYSTEM_INTEGRITY_SUPPLEMENTAL_POLICY_NOT_AUTHORIZED = 0xc0e90006,
    STATUS_NO_APPLICABLE_APP_LICENSES_FOUND = 0xc0ea0001,
    STATUS_CLIP_LICENSE_NOT_FOUND = 0xc0ea0002,
    STATUS_CLIP_DEVICE_LICENSE_MISSING = 0xc0ea0003,
    STATUS_CLIP_LICENSE_INVALID_SIGNATURE = 0xc0ea0004,
    STATUS_CLIP_KEYHOLDER_LICENSE_MISSING_OR_INVALID = 0xc0ea0005,
    STATUS_CLIP_LICENSE_EXPIRED = 0xc0ea0006,
    STATUS_CLIP_LICENSE_SIGNED_BY_UNKNOWN_SOURCE = 0xc0ea0007,
    STATUS_CLIP_LICENSE_NOT_SIGNED = 0xc0ea0008,
    STATUS_CLIP_LICENSE_HARDWARE_ID_OUT_OF_TOLERANCE = 0xc0ea0009,
    STATUS_CLIP_LICENSE_DEVICE_ID_MISMATCH = 0xc0ea000a,
    STATUS_PLATFORM_MANIFEST_NOT_AUTHORIZED = 0xc0eb0001,
    STATUS_PLATFORM_MANIFEST_INVALID = 0xc0eb0002,
    STATUS_PLATFORM_MANIFEST_FILE_NOT_AUTHORIZED = 0xc0eb0003,
    STATUS_PLATFORM_MANIFEST_CATALOG_NOT_AUTHORIZED = 0xc0eb0004,
    STATUS_PLATFORM_MANIFEST_BINARY_ID_NOT_FOUND = 0xc0eb0005,
    STATUS_PLATFORM_MANIFEST_NOT_ACTIVE = 0xc0eb0006,
    STATUS_PLATFORM_MANIFEST_NOT_SIGNED = 0xc0eb0007,
    STATUS_APPEXEC_CONDITION_NOT_SATISFIED = 0xc0ec0000,
    STATUS_APPEXEC_HANDLE_INVALIDATED = 0xc0ec0001,
    STATUS_APPEXEC_INVALID_HOST_GENERATION = 0xc0ec0002,
    STATUS_APPEXEC_UNEXPECTED_PROCESS_REGISTRATION = 0xc0ec0003,
    STATUS_APPEXEC_INVALID_HOST_STATE = 0xc0ec0004,
    STATUS_APPEXEC_NO_DONOR = 0xc0ec0005,
    STATUS_APPEXEC_HOST_ID_MISMATCH = 0xc0ec0006,
    STATUS_APPEXEC_UNKNOWN_USER = 0xc0ec0007,
}

/// Values for [`MINIDUMP_EXCEPTION::exception_information`]`[0]`,
/// when [`MINIDUMP_EXCEPTION::exception_code`] is [`NtStatusWindows::STATUS_STACK_BUFFER_OVERRUN`].
/// This describes the underlying reason for the crash.
///
/// The values were generated from from winnt.h in the Windows 10 SDK
/// (version 10.0.19041.0) using the following script:
/// ```sh
/// egrep '#define FAST_FAIL_[A-Z_0-9]+\s+[0-9]' winnt.h \
/// | tr -d '\r' \
/// | sed -r 's@#define (FAST_FAIL_[A-Z_0-9]+)\s+([0-9]+).*@\2 \1@' \
/// | sed -r 's@([0-9]+) ([A-Z_0-9]+)@    \2 = \1,@'
/// ```
#[repr(u64)]
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum FastFailCode {
    FAST_FAIL_LEGACY_GS_VIOLATION = 0,
    FAST_FAIL_VTGUARD_CHECK_FAILURE = 1,
    FAST_FAIL_STACK_COOKIE_CHECK_FAILURE = 2,
    FAST_FAIL_CORRUPT_LIST_ENTRY = 3,
    FAST_FAIL_INCORRECT_STACK = 4,
    FAST_FAIL_INVALID_ARG = 5,
    FAST_FAIL_GS_COOKIE_INIT = 6,
    FAST_FAIL_FATAL_APP_EXIT = 7,
    FAST_FAIL_RANGE_CHECK_FAILURE = 8,
    FAST_FAIL_UNSAFE_REGISTRY_ACCESS = 9,
    FAST_FAIL_GUARD_ICALL_CHECK_FAILURE = 10,
    FAST_FAIL_GUARD_WRITE_CHECK_FAILURE = 11,
    FAST_FAIL_INVALID_FIBER_SWITCH = 12,
    FAST_FAIL_INVALID_SET_OF_CONTEXT = 13,
    FAST_FAIL_INVALID_REFERENCE_COUNT = 14,
    FAST_FAIL_INVALID_JUMP_BUFFER = 18,
    FAST_FAIL_MRDATA_MODIFIED = 19,
    FAST_FAIL_CERTIFICATION_FAILURE = 20,
    FAST_FAIL_INVALID_EXCEPTION_CHAIN = 21,
    FAST_FAIL_CRYPTO_LIBRARY = 22,
    FAST_FAIL_INVALID_CALL_IN_DLL_CALLOUT = 23,
    FAST_FAIL_INVALID_IMAGE_BASE = 24,
    FAST_FAIL_DLOAD_PROTECTION_FAILURE = 25,
    FAST_FAIL_UNSAFE_EXTENSION_CALL = 26,
    FAST_FAIL_DEPRECATED_SERVICE_INVOKED = 27,
    FAST_FAIL_INVALID_BUFFER_ACCESS = 28,
    FAST_FAIL_INVALID_BALANCED_TREE = 29,
    FAST_FAIL_INVALID_NEXT_THREAD = 30,
    FAST_FAIL_GUARD_ICALL_CHECK_SUPPRESSED = 31,
    FAST_FAIL_APCS_DISABLED = 32,
    FAST_FAIL_INVALID_IDLE_STATE = 33,
    FAST_FAIL_MRDATA_PROTECTION_FAILURE = 34,
    FAST_FAIL_UNEXPECTED_HEAP_EXCEPTION = 35,
    FAST_FAIL_INVALID_LOCK_STATE = 36,
    FAST_FAIL_GUARD_JUMPTABLE = 37,
    FAST_FAIL_INVALID_LONGJUMP_TARGET = 38,
    FAST_FAIL_INVALID_DISPATCH_CONTEXT = 39,
    FAST_FAIL_INVALID_THREAD = 40,
    FAST_FAIL_INVALID_SYSCALL_NUMBER = 41,
    FAST_FAIL_INVALID_FILE_OPERATION = 42,
    FAST_FAIL_LPAC_ACCESS_DENIED = 43,
    FAST_FAIL_GUARD_SS_FAILURE = 44,
    FAST_FAIL_LOADER_CONTINUITY_FAILURE = 45,
    FAST_FAIL_GUARD_EXPORT_SUPPRESSION_FAILURE = 46,
    FAST_FAIL_INVALID_CONTROL_STACK = 47,
    FAST_FAIL_SET_CONTEXT_DENIED = 48,
    FAST_FAIL_INVALID_IAT = 49,
    FAST_FAIL_HEAP_METADATA_CORRUPTION = 50,
    FAST_FAIL_PAYLOAD_RESTRICTION_VIOLATION = 51,
    FAST_FAIL_LOW_LABEL_ACCESS_DENIED = 52,
    FAST_FAIL_ENCLAVE_CALL_FAILURE = 53,
    FAST_FAIL_UNHANDLED_LSS_EXCEPTON = 54,
    FAST_FAIL_ADMINLESS_ACCESS_DENIED = 55,
    FAST_FAIL_UNEXPECTED_CALL = 56,
    FAST_FAIL_CONTROL_INVALID_RETURN_ADDRESS = 57,
    FAST_FAIL_UNEXPECTED_HOST_BEHAVIOR = 58,
    FAST_FAIL_FLAGS_CORRUPTION = 59,
    FAST_FAIL_VEH_CORRUPTION = 60,
    FAST_FAIL_ETW_CORRUPTION = 61,
    FAST_FAIL_RIO_ABORT = 62,
    FAST_FAIL_INVALID_PFN = 63,
}

/// The different kinds of EXCEPTION_ACCESS_VIOLATION.
///
/// These constants are defined in the [MSDN documentation][msdn] of
/// the EXCEPTION_RECORD structure.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-exception_record
#[repr(u64)]
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeWindowsAccessType {
    READ = 0,
    WRITE = 1,
    EXEC = 8,
}

/// The different kinds of EXCEPTION_IN_PAGE_ERROR.
///
/// These constants are defined in the [MSDN documentation][msdn] of
/// the EXCEPTION_RECORD structure.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-exception_record
#[repr(u64)]
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeWindowsInPageErrorType {
    READ = 0,
    WRITE = 1,
    EXEC = 8,
}

/// Values for [`MINIDUMP_EXCEPTION::exception_code`] for crashes on Linux
///
/// These are primarily signal numbers from bits/signum.h.
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeLinux {
    /// Hangup (POSIX)
    SIGHUP = 0x1u32,
    /// Interrupt (ANSI)
    SIGINT = 0x2,
    /// Quit (POSIX)
    SIGQUIT = 0x3,
    /// Illegal instruction (ANSI)
    SIGILL = 0x4,
    /// Trace trap (POSIX)
    SIGTRAP = 0x5,
    /// Abort (ANSI)
    SIGABRT = 0x6,
    /// BUS error (4.2 BSD)
    SIGBUS = 0x7,
    /// Floating-point exception (ANSI)
    SIGFPE = 0x8,
    /// Kill, unblockable (POSIX)
    SIGKILL = 0x9,
    /// User-defined signal 1 (POSIX)
    SIGUSR1 = 0xa,
    /// Segmentation violation (ANSI)
    SIGSEGV = 0xb,
    /// User-defined signal 2 (POSIX)
    SIGUSR2 = 0xc,
    /// Broken pipe (POSIX)
    SIGPIPE = 0xd,
    /// Alarm clock (POSIX)
    SIGALRM = 0xe,
    /// Termination (ANSI)
    SIGTERM = 0xf,
    /// Stack fault
    SIGSTKFLT = 0x10,
    /// Child status has changed (POSIX)
    SIGCHLD = 0x11,
    /// Continue (POSIX)
    SIGCONT = 0x12,
    /// Stop, unblockable (POSIX)
    SIGSTOP = 0x13,
    /// Keyboard stop (POSIX)
    SIGTSTP = 0x14,
    /// Background read from tty (POSIX)
    SIGTTIN = 0x15,
    /// Background write to tty (POSIX)
    SIGTTOU = 0x16,
    /// Urgent condition on socket (4.2 BSD)
    SIGURG = 0x17,
    /// CPU limit exceeded (4.2 BSD)
    SIGXCPU = 0x18,
    /// File size limit exceeded (4.2 BSD)
    SIGXFSZ = 0x19,
    /// Virtual alarm clock (4.2 BSD)
    SIGVTALRM = 0x1a,
    /// Profiling alarm clock (4.2 BSD)
    SIGPROF = 0x1b,
    /// Window size change (4.3 BSD, Sun)
    SIGWINCH = 0x1c,
    /// I/O now possible (4.2 BSD)
    SIGIO = 0x1d,
    /// Power failure restart (System V)
    SIGPWR = 0x1e,
    /// Bad system call
    SIGSYS = 0x1f,
    /// No exception, dump requested
    DUMP_REQUESTED = 0xffffffff,
}

// These values come from asm-generic/siginfo.h
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeLinuxSigillKind {
    SI_USER = 0,
    ILL_ILLOPC = 1,
    ILL_ILLOPN = 2,
    ILL_ILLADR = 3,
    ILL_ILLTRP = 4,
    ILL_PRVOPC = 5,
    ILL_PRVREG = 6,
    ILL_COPROC = 7,
    ILL_BADSTK = 8,
    SI_KERNEL = 0x80,
}

#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeLinuxSigfpeKind {
    SI_USER = 0,
    FPE_INTDIV = 1,
    FPE_INTOVF = 2,
    FPE_FLTDIV = 3,
    FPE_FLTOVF = 4,
    FPE_FLTUND = 5,
    FPE_FLTRES = 6,
    FPE_FLTINV = 7,
    FPE_FLTSUB = 8,
    SI_KERNEL = 0x80,
}

#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeLinuxSigsegvKind {
    SI_USER = 0,
    SEGV_MAPERR = 1,
    SEGV_ACCERR = 2,
    SEGV_BNDERR = 3,
    SEGV_PKUERR = 4,
    SI_KERNEL = 0x80,
}

#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeLinuxSigbusKind {
    SI_USER = 0,
    BUS_ADRALN = 1,
    BUS_ADRERR = 2,
    BUS_OBJERR = 3,
    BUS_MCEERR_AR = 4,
    BUS_MCEERR_AO = 5,
    SI_KERNEL = 0x80,
}

/// Values for [`MINIDUMP_EXCEPTION::exception_code`] for crashes on macOS
///
/// Based on Darwin/macOS' mach/exception_types.h. This is what macOS calls an "exception",
/// not a "code".
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeMac {
    /// code can be a kern_return_t
    EXC_BAD_ACCESS = 1,
    /// code is CPU-specific
    EXC_BAD_INSTRUCTION = 2,
    /// code is CPU-specific
    EXC_ARITHMETIC = 3,
    /// code is CPU-specific
    EXC_EMULATION = 4,
    EXC_SOFTWARE = 5,
    /// code is CPU-specific
    EXC_BREAKPOINT = 6,
    EXC_SYSCALL = 7,
    EXC_MACH_SYSCALL = 8,
    EXC_RPC_ALERT = 9,
    /// Fake exception code used by Crashpad's SimulateCrash ('CPsx')
    SIMULATED = 0x43507378,
}

// These error codes are based on
// * mach/ppc/exception.h
// * mach/i386/exception.h

/// Mac/iOS Kernel Bad Access Exceptions
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeMacBadAccessKernType {
    // These are relevant kern_return_t values from mach/kern_return.h
    KERN_INVALID_ADDRESS = 1,
    KERN_PROTECTION_FAILURE = 2,
    KERN_NO_ACCESS = 8,
    KERN_MEMORY_FAILURE = 9,
    KERN_MEMORY_ERROR = 10,
    KERN_CODESIGN_ERROR = 50,
}

/// Mac/iOS Arm Userland Bad Accesses Exceptions
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeMacBadAccessArmType {
    EXC_ARM_DA_ALIGN = 0x0101,
    EXC_ARM_DA_DEBUG = 0x0102,
}

/// Mac/iOS Ppc Userland Bad Access Exceptions
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeMacBadAccessPpcType {
    EXC_PPC_VM_PROT_READ = 0x0101,
    EXC_PPC_BADSPACE = 0x0102,
    EXC_PPC_UNALIGNED = 0x0103,
}

/// Mac/iOS x86 Userland Bad Access Exceptions
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeMacBadAccessX86Type {
    EXC_I386_GPFLT = 13,
}

/// Mac/iOS Arm Bad Instruction Exceptions
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeMacBadInstructionArmType {
    EXC_ARM_UNDEFINED = 1,
}

/// Mac/iOS Ppc Bad Instruction Exceptions
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeMacBadInstructionPpcType {
    EXC_PPC_INVALID_SYSCALL = 1,
    EXC_PPC_UNIPL_INST = 2,
    EXC_PPC_PRIVINST = 3,
    EXC_PPC_PRIVREG = 4,
    EXC_PPC_TRACE = 5,
    EXC_PPC_PERFMON = 6,
}

/// Mac/iOS x86 Bad Instruction Exceptions
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeMacBadInstructionX86Type {
    /// Invalid Operation
    EXC_I386_INVOP = 1,

    // The rest of these are raw x86 interrupt codes.
    /// Invalid Task State Segment
    EXC_I386_INVTSSFLT = 10,
    /// Segment Not Present
    EXC_I386_SEGNPFLT = 11,
    /// Stack Fault
    EXC_I386_STKFLT = 12,
    /// General Protection Fault
    EXC_I386_GPFLT = 13,
    /// Alignment Fault
    EXC_I386_ALIGNFLT = 17,
    // For sake of completeness, here's the interrupt codes that won't show up here (and why):

    // EXC_I386_DIVERR    =  0: mapped to EXC_ARITHMETIC/EXC_I386_DIV
    // EXC_I386_SGLSTP    =  1: mapped to EXC_BREAKPOINT/EXC_I386_SGL
    // EXC_I386_NMIFLT    =  2: should not occur in user space
    // EXC_I386_BPTFLT    =  3: mapped to EXC_BREAKPOINT/EXC_I386_BPT
    // EXC_I386_INTOFLT   =  4: mapped to EXC_ARITHMETIC/EXC_I386_INTO
    // EXC_I386_BOUNDFLT  =  5: mapped to EXC_ARITHMETIC/EXC_I386_BOUND
    // EXC_I386_INVOPFLT  =  6: mapped to EXC_BAD_INSTRUCTION/EXC_I386_INVOP
    // EXC_I386_NOEXTFLT  =  7: should be handled by the kernel
    // EXC_I386_DBLFLT    =  8: should be handled (if possible) by the kernel
    // EXC_I386_EXTOVRFLT =  9: mapped to EXC_BAD_ACCESS/(PROT_READ|PROT_EXEC)
    // EXC_I386_PGFLT     = 14: should not occur in user space
    // EXC_I386_EXTERRFLT = 16: mapped to EXC_ARITHMETIC/EXC_I386_EXTERR
    // EXC_I386_ENOEXTFLT = 32: should be handled by the kernel
    // EXC_I386_ENDPERR   = 33: should not occur
}

/// Mac/iOS Ppc Arithmetic Exceptions
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeMacArithmeticPpcType {
    /// Integer ovrflow
    EXC_PPC_OVERFLOW = 1,
    /// Integer Divide-By-Zero
    EXC_PPC_ZERO_DIVIDE = 2,
    /// Float Inexact
    EXC_FLT_INEXACT = 3,
    /// Float Divide-By-Zero
    EXC_PPC_FLT_ZERO_DIVIDE = 4,
    /// Float Underflow
    EXC_PPC_FLT_UNDERFLOW = 5,
    /// Float Overflow
    EXC_PPC_FLT_OVERFLOW = 6,
    /// Float Not A Number
    EXC_PPC_FLT_NOT_A_NUMBER = 7,

    // NOTE: comments in breakpad suggest these two are actually supposed to be
    // for ExceptionCodeMac::EXC_EMULATION, but for now let's duplicate breakpad.
    EXC_PPC_NOEMULATION = 8,
    EXC_PPC_ALTIVECASSIST = 9,
}

/// Mac/iOS x86 Arithmetic Exceptions
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeMacArithmeticX86Type {
    EXC_I386_DIV = 1,
    EXC_I386_INTO = 2,
    EXC_I386_NOEXT = 3,
    EXC_I386_EXTOVR = 4,
    EXC_I386_EXTERR = 5,
    EXC_I386_EMERR = 6,
    EXC_I386_BOUND = 7,
    EXC_I386_SSEEXTERR = 8,
}

/// Mac/iOS "Software" Exceptions
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeMacSoftwareType {
    SIGABRT = 0x00010002u32,
    UNCAUGHT_NS_EXCEPTION = 0xDEADC0DE,
    EXC_PPC_TRAP = 0x00000001,
    EXC_PPC_MIGRATE = 0x00010100,
    // Breakpad also defines these doesn't use them for Software crashes
    // SIGSYS  = 0x00010000,
    // SIGPIPE = 0x00010001,
}

/// Mac/iOS Arm Breakpoint Exceptions
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeMacBreakpointArmType {
    EXC_ARM_DA_ALIGN = 0x0101,
    EXC_ARM_DA_DEBUG = 0x0102,
    EXC_ARM_BREAKPOINT = 1,
}

/// Mac/iOS Ppc Breakpoint Exceptions
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeMacBreakpointPpcType {
    EXC_PPC_BREAKPOINT = 1,
}

/// Mac/iOS x86 Breakpoint Exceptions
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ExceptionCodeMacBreakpointX86Type {
    EXC_I386_SGL = 1,
    EXC_I386_BPT = 2,
}

/// Valid bits in a `context_flags` for [`ContextFlagsCpu`]
pub const CONTEXT_CPU_MASK: u32 = 0xffffff00;

bitflags! {
    /// CPU type values in the `context_flags` member of `CONTEXT_` structs
    ///
    /// This applies to the [`CONTEXT_ARM`], [`CONTEXT_PPC`], [`CONTEXT_MIPS`],
    /// [`CONTEXT_AMD64`], [`CONTEXT_ARM64`], [`CONTEXT_PPC64`], [`CONTEXT_SPARC`] and
    /// [`CONTEXT_ARM64_OLD`] structs.
    pub struct ContextFlagsCpu: u32 {
        const CONTEXT_IA64 = 0x80000;
        /// Super-H, includes SH3, from winnt.h in the Windows CE 5.0 SDK
        const CONTEXT_SHX = 0xc0;
        /// From winnt.h in the Windows CE 5.0 SDK, no longer used
        ///
        /// Originally used by Breakpad but changed after conflicts with other context
        /// flag bits.
        const CONTEXT_ARM_OLD = 0x40;
        /// Alpha, from winnt.h in the Windows CE 5.0 SDK
        const CONTEXT_ALPHA = 0x20000;
        const CONTEXT_AMD64 = 0x100000;
        const CONTEXT_ARM = 0x40000000;
        const CONTEXT_ARM64 = 0x400000;
        const CONTEXT_ARM64_OLD = 0x80000000;
        const CONTEXT_MIPS = 0x40000;
        const CONTEXT_MIPS64 = 0x80000;
        const CONTEXT_PPC = 0x20000000;
        const CONTEXT_PPC64 = 0x1000000;
        const CONTEXT_SPARC = 0x10000000;
        const CONTEXT_X86 = 0x10000;
    }
}

impl ContextFlagsCpu {
    /// Populate a [`ContextFlagsCpu`] with valid bits from `flags`
    pub fn from_flags(flags: u32) -> ContextFlagsCpu {
        ContextFlagsCpu::from_bits_truncate(flags & CONTEXT_CPU_MASK)
    }
}

/// Possible contents of [`CONTEXT_AMD64::float_save`].
///
/// This struct matches the definition of the struct with the same name from WinNT.h.
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct XMM_SAVE_AREA32 {
    pub control_word: u16,
    pub status_word: u16,
    pub tag_word: u8,
    pub reserved1: u8,
    pub error_opcode: u16,
    pub error_offset: u32,
    pub error_selector: u16,
    pub reserved2: u16,
    pub data_offset: u32,
    pub data_selector: u16,
    pub reserved3: u16,
    pub mx_csr: u32,
    pub mx_csr_mask: u32,
    pub float_registers: [u128; 8],
    pub xmm_registers: [u128; 16],
    pub reserved4: [u8; 96],
}

/// Possible contents of [`CONTEXT_AMD64::float_save`].
///
/// This is defined as an anonymous struct inside an anonymous union in
/// the x86-64 CONTEXT struct in WinNT.h.
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct SSE_REGISTERS {
    pub header: [u128; 2],
    pub legacy: [u128; 8],
    pub xmm0: u128,
    pub xmm1: u128,
    pub xmm2: u128,
    pub xmm3: u128,
    pub xmm4: u128,
    pub xmm5: u128,
    pub xmm6: u128,
    pub xmm7: u128,
    pub xmm8: u128,
    pub xmm9: u128,
    pub xmm10: u128,
    pub xmm11: u128,
    pub xmm12: u128,
    pub xmm13: u128,
    pub xmm14: u128,
    pub xmm15: u128,
}

/// An x86-64 (amd64) CPU context
///
/// This struct matches the definition of `CONTEXT` in WinNT.h for x86-64.
#[derive(Debug, SmartDefault, Clone, Pread, SizeWith)]
pub struct CONTEXT_AMD64 {
    pub p1_home: u64,
    pub p2_home: u64,
    pub p3_home: u64,
    pub p4_home: u64,
    pub p5_home: u64,
    pub p6_home: u64,
    pub context_flags: u32,
    pub mx_csr: u32,
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub fs: u16,
    pub gs: u16,
    pub ss: u16,
    pub eflags: u32,
    pub dr0: u64,
    pub dr1: u64,
    pub dr2: u64,
    pub dr3: u64,
    pub dr6: u64,
    pub dr7: u64,
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbx: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    /// Floating point state
    ///
    /// This is defined as a union in the C headers, but also
    /// ` MAXIMUM_SUPPORTED_EXTENSION` is defined as 512 bytes.
    ///
    /// Callers that want to access the underlying data can use [`Pread`] to read either
    /// an [`XMM_SAVE_AREA32`] or [`SSE_REGISTERS`] struct from this raw data as appropriate.
    #[default([0; 512])]
    pub float_save: [u8; 512],
    #[default([0; 26])]
    pub vector_register: [u128; 26],
    pub vector_control: u64,
    pub debug_control: u64,
    pub last_branch_to_rip: u64,
    pub last_branch_from_rip: u64,
    pub last_exception_to_rip: u64,
    pub last_exception_from_rip: u64,
}

/// ARM floating point state
#[derive(Debug, Clone, Default, Pread, SizeWith)]
pub struct FLOATING_SAVE_AREA_ARM {
    pub fpscr: u64,
    pub regs: [u64; 32],
    pub extra: [u32; 8],
}

/// An ARM CPU context
///
/// This is a Breakpad extension, and does not match the definition of `CONTEXT` for ARM
/// in WinNT.h.
#[derive(Debug, Clone, Default, Pread, SizeWith)]
pub struct CONTEXT_ARM {
    pub context_flags: u32,
    pub iregs: [u32; 16],
    pub cpsr: u32,
    pub float_save: FLOATING_SAVE_AREA_ARM,
}

/// Offsets into [`CONTEXT_ARM::iregs`] for registers with a dedicated or conventional purpose
#[repr(usize)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ArmRegisterNumbers {
    IosFramePointer = 7,
    FramePointer = 11,
    StackPointer = 13,
    LinkRegister = 14,
    ProgramCounter = 15,
}

impl ArmRegisterNumbers {
    pub const fn name(self) -> &'static str {
        match self {
            Self::IosFramePointer => "r7",
            Self::FramePointer => "r11",
            Self::StackPointer => "r13",
            Self::LinkRegister => "r14",
            Self::ProgramCounter => "r15",
        }
    }
}

/// aarch64 floating point state (old)
#[derive(Debug, Clone, Copy, Default, Pread, SizeWith)]
pub struct FLOATING_SAVE_AREA_ARM64_OLD {
    pub fpsr: u32,
    pub fpcr: u32,
    pub regs: [u128; 32usize],
}

/// An old aarch64 (arm64) CPU context
///
/// This is a Breakpad extension.
#[derive(Debug, Clone, Copy, Default, Pread, SizeWith)]
#[repr(packed)]
pub struct CONTEXT_ARM64_OLD {
    pub context_flags: u64,
    pub iregs: [u64; 32],
    pub pc: u64,
    pub cpsr: u32,
    pub float_save: FLOATING_SAVE_AREA_ARM64_OLD,
}

/// aarch64 floating point state
#[derive(Debug, Clone, Default, Pread, SizeWith)]
pub struct FLOATING_SAVE_AREA_ARM64 {
    pub regs: [u128; 32usize],
    pub fpsr: u32,
    pub fpcr: u32,
}

/// An aarch64 (arm64) CPU context
///
/// This is a Breakpad extension, and does not match the definition of `CONTEXT` for aarch64
/// in WinNT.h.
#[derive(Debug, Default, Clone, Pread, SizeWith)]
pub struct CONTEXT_ARM64 {
    pub context_flags: u32,
    pub cpsr: u32,
    pub iregs: [u64; 32],
    pub pc: u64,
    pub float_save: FLOATING_SAVE_AREA_ARM64,
    pub bcr: [u32; 8],
    pub bvr: [u64; 8],
    pub wcr: [u32; 2],
    pub wvr: [u64; 2],
}

/// Offsets into [`CONTEXT_ARM64::iregs`] for registers with a dedicated or conventional purpose
#[repr(usize)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Arm64RegisterNumbers {
    FramePointer = 29,
    LinkRegister = 30,
    StackPointer = 31,
    ProgramCounter = 32,
}

impl Arm64RegisterNumbers {
    pub const fn name(self) -> &'static str {
        match self {
            Self::FramePointer => "x29",
            Self::LinkRegister => "x30",
            Self::StackPointer => "sp",
            Self::ProgramCounter => "pc",
        }
    }
}

/// MIPS floating point state
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct FLOATING_SAVE_AREA_MIPS {
    pub regs: [u64; 32],
    pub fpcsr: u32,
    pub fir: u32,
}

/// A MIPS CPU context
///
/// This is a Breakpad extension, as there is no definition of `CONTEXT` for MIPS in WinNT.h.
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct CONTEXT_MIPS {
    pub context_flags: u32,
    pub _pad0: u32,
    pub iregs: [u64; 32],
    pub mdhi: u64,
    pub mdlo: u64,
    pub hi: [u32; 3],
    pub lo: [u32; 3],
    pub dsp_control: u32,
    pub _pad1: u32,
    pub epc: u64,
    pub badvaddr: u64,
    pub status: u32,
    pub cause: u32,
    pub float_save: FLOATING_SAVE_AREA_MIPS,
}

/// Offsets into [`CONTEXT_MIPS::iregs`] for registers with a dedicated or conventional purpose
#[repr(usize)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MipsRegisterNumbers {
    S0 = 16,
    S1 = 17,
    S2 = 18,
    S3 = 19,
    S4 = 20,
    S5 = 21,
    S6 = 22,
    S7 = 23,
    GlobalPointer = 28,
    StackPointer = 29,
    FramePointer = 30,
    ReturnAddress = 31,
}

/// PPC floating point state
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct FLOATING_SAVE_AREA_PPC {
    pub fpregs: [u64; 32],
    pub fpscr_pad: u32,
    pub fpscr: u32,
}

/// PPC vector state
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct VECTOR_SAVE_AREA_PPC {
    pub save_vr: [u128; 32],
    pub save_vscr: u128,
    pub save_pad5: [u32; 4],
    pub save_vrvalid: u32,
    pub save_pad6: [u32; 7],
}

/// A PPC CPU context
///
/// This is a Breakpad extension, as there is no definition of `CONTEXT` for PPC in WinNT.h.
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct CONTEXT_PPC {
    pub context_flags: u32,
    pub srr0: u32,
    pub srr1: u32,
    pub gpr: [u32; 32],
    pub cr: u32,
    pub xer: u32,
    pub lr: u32,
    pub ctr: u32,
    pub mq: u32,
    pub vrsave: u32,
    pub float_save: FLOATING_SAVE_AREA_PPC,
    pub vector_save: VECTOR_SAVE_AREA_PPC,
}

/// Offsets into [`CONTEXT_PPC::gpr`] for registers with a dedicated or conventional purpose
#[repr(usize)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PpcRegisterNumbers {
    StackPointer = 1,
}

/// A PPC64 CPU context
///
/// This is a Breakpad extension, as there is no definition of `CONTEXT` for PPC64 in WinNT.h.
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct CONTEXT_PPC64 {
    pub context_flags: u64,
    pub srr0: u64,
    pub srr1: u64,
    pub gpr: [u64; 32],
    pub cr: u64,
    pub xer: u64,
    pub lr: u64,
    pub ctr: u64,
    pub vrsave: u64,
    pub float_save: FLOATING_SAVE_AREA_PPC,
    pub vector_save: VECTOR_SAVE_AREA_PPC,
}

/// Offsets into [`CONTEXT_PPC64::gpr`] for registers with a dedicated or conventional purpose
#[repr(usize)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Ppc64RegisterNumbers {
    StackPointer = 1,
}

/// SPARC floating point state
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct FLOATING_SAVE_AREA_SPARC {
    pub regs: [u64; 32],
    pub filler: u64,
    pub fsr: u64,
}

/// A SPARC CPU context
///
/// This is a Breakpad extension, as there is no definition of `CONTEXT` for SPARC in WinNT.h.
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct CONTEXT_SPARC {
    pub context_flags: u32,
    pub flag_pad: u32,
    pub g_r: [u64; 32],
    pub ccr: u64,
    pub pc: u64,
    pub npc: u64,
    pub y: u64,
    pub asi: u64,
    pub fprs: u64,
    pub float_save: FLOATING_SAVE_AREA_SPARC,
}

/// Offsets into [`CONTEXT_SPARC::g_r`] for registers with a dedicated or conventional purpose
#[repr(usize)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SparcRegisterNumbers {
    StackPointer = 14,
}

/// x86 floating point state
///
/// This struct matches the definition of the `FLOATING_SAVE_AREA` struct from WinNT.h.
#[derive(Debug, Clone, SmartDefault, Pread, SizeWith)]
pub struct FLOATING_SAVE_AREA_X86 {
    pub control_word: u32,
    pub status_word: u32,
    pub tag_word: u32,
    pub error_offset: u32,
    pub error_selector: u32,
    pub data_offset: u32,
    pub data_selector: u32,
    #[default([0; 80])]
    pub register_area: [u8; 80], // SIZE_OF_80387_REGISTERS
    pub cr0_npx_state: u32,
}

/// An x86 CPU context
///
/// This struct matches the definition of `CONTEXT` in WinNT.h for x86.
#[derive(Debug, Clone, SmartDefault, Pread, SizeWith)]
pub struct CONTEXT_X86 {
    pub context_flags: u32,
    pub dr0: u32,
    pub dr1: u32,
    pub dr2: u32,
    pub dr3: u32,
    pub dr6: u32,
    pub dr7: u32,
    pub float_save: FLOATING_SAVE_AREA_X86,
    pub gs: u32,
    pub fs: u32,
    pub es: u32,
    pub ds: u32,
    pub edi: u32,
    pub esi: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub ebp: u32,
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    pub esp: u32,
    pub ss: u32,
    #[default([0; 512])]
    pub extended_registers: [u8; 512], // MAXIMUM_SUPPORTED_EXTENSION
}

/// CPU information contained within the [`MINIDUMP_SYSTEM_INFO`] struct
///
/// This struct matches the definition of the `CPU_INFORMATION` union from minidumpapiset.h.
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct CPU_INFORMATION {
    /// `data` is defined as a union in the Microsoft headers
    ///
    /// It is the union of [`X86CpuInfo`], [`ARMCpuInfo`] (Breakpad-specific), and
    /// [`OtherCpuInfo`] defined below. It does not seem possible to safely derive [`Pread`]
    /// on an actual union, so we provide the raw data here and expect callers to use
    /// [`Pread`] to derive the specific union representation desired.
    pub data: [u8; 24],
}

/// x86-specific CPU information derived from the `cpuid` instruction
///
/// This struct matches the definition of the struct of the same name from minidumpapiset.h,
/// which is contained within the [`CPU_INFORMATION`] union.
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct X86CpuInfo {
    pub vendor_id: [u32; 3],
    pub version_information: u32,
    pub feature_information: u32,
    pub amd_extended_cpu_features: u32,
}

/// Arm-specific CPU information (Breakpad extension)
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct ARMCpuInfo {
    pub cpuid: u32,
    /// Hardware capabilities
    ///
    /// See [`ArmElfHwCaps`] for possible values.
    pub elf_hwcaps: u32,
}

/// CPU information for non-x86 CPUs
///
/// This struct matches the definition of the struct of the same name from minidumpapiset.h,
/// which is contained within the [`CPU_INFORMATION`] union.
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct OtherCpuInfo {
    pub processor_features: [u64; 2],
}

/// Processor and operating system information
///
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_system_info
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct MINIDUMP_SYSTEM_INFO {
    /// The system's processor architecture
    ///
    /// Known values are defined in [`ProcessorArchitecture`].
    pub processor_architecture: u16,
    /// x86 (5 = 586, 6 = 686 ...) or ARM (6 = ARMv6, 7 = ARMv7 ...) CPU level
    pub processor_level: u16,
    /// For x86, 0xMMSS where MM=model, SS=stepping
    pub processor_revision: u16,
    pub number_of_processors: u8,
    pub product_type: u8,
    pub major_version: u32,
    pub minor_version: u32,
    pub build_number: u32,
    /// The operating system platform
    ///
    /// Known values are defined in [`PlatformId`].
    pub platform_id: u32,
    pub csd_version_rva: RVA,
    pub suite_mask: u16,
    pub reserved2: u16,
    pub cpu: CPU_INFORMATION,
}

/// Known values of [`MINIDUMP_SYSTEM_INFO::processor_architecture`]
///
/// Many of these are taken from definitions in WinNT.h, but several of them are
/// Breakpad extensions.
#[repr(u16)]
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum ProcessorArchitecture {
    PROCESSOR_ARCHITECTURE_INTEL = 0,
    PROCESSOR_ARCHITECTURE_MIPS = 1,
    PROCESSOR_ARCHITECTURE_ALPHA = 2,
    PROCESSOR_ARCHITECTURE_PPC = 3,
    PROCESSOR_ARCHITECTURE_SHX = 4,
    PROCESSOR_ARCHITECTURE_ARM = 5,
    PROCESSOR_ARCHITECTURE_IA64 = 6,
    PROCESSOR_ARCHITECTURE_ALPHA64 = 7,
    /// Microsoft Intermediate Language
    PROCESSOR_ARCHITECTURE_MSIL = 8,
    PROCESSOR_ARCHITECTURE_AMD64 = 9,
    /// WoW64
    PROCESSOR_ARCHITECTURE_IA32_ON_WIN64 = 10,
    PROCESSOR_ARCHITECTURE_ARM64 = 12,
    /// Breakpad-defined value for SPARC
    PROCESSOR_ARCHITECTURE_SPARC = 0x8001,
    /// Breakpad-defined value for PPC64
    PROCESSOR_ARCHITECTURE_PPC64 = 0x8002,
    /// Breakpad-defined value for ARM64
    PROCESSOR_ARCHITECTURE_ARM64_OLD = 0x8003,
    /// Breakpad-defined value for MIPS64
    PROCESSOR_ARCHITECTURE_MIPS64 = 0x8004,
    PROCESSOR_ARCHITECTURE_UNKNOWN = 0xffff,
}

/// Known values of [`MINIDUMP_SYSTEM_INFO::platform_id`]
///
/// The Windows values here are taken from defines in WinNT.h, but the rest are Breakpad
/// extensions.
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum PlatformId {
    /// Windows 3.1
    VER_PLATFORM_WIN32s = 1,
    /// Windows 95-98-Me
    VER_PLATFORM_WIN32_WINDOWS = 2,
    /// Windows NT, 2000+
    VER_PLATFORM_WIN32_NT = 3,
    /// Windows CE, Windows Mobile
    VER_PLATFORM_WIN32_CE = 4,
    /// Generic Unix-ish (Breakpad extension)
    Unix = 0x8000,
    /// macOS/Darwin (Breakpad extension)
    MacOs = 0x8101,
    /// iOS (Breakpad extension)
    Ios = 0x8102,
    /// Linux (Breakpad extension)
    Linux = 0x8201,
    /// Solaris (Breakpad extension)
    Solaris = 0x8202,
    /// Android (Breakpad extension)
    Android = 0x8203,
    /// PlayStation 3 (Breakpad extension)
    Ps3 = 0x8204,
    /// Native Client (Breakpad extension)
    NaCl = 0x8205,
}

/// A date and time
///
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://msdn.microsoft.com/en-us/library/windows/desktop/ms724950(v=vs.85).aspx
#[derive(Debug, Clone, Default, Pread, SizeWith, PartialEq, Eq)]
pub struct SYSTEMTIME {
    pub year: u16,
    pub month: u16,
    pub day_of_week: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milliseconds: u16,
}

/// Settings for a time zone
///
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/timezoneapi/ns-timezoneapi-_time_zone_information
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct TIME_ZONE_INFORMATION {
    pub bias: i32,
    pub standard_name: [u16; 32],
    pub standard_date: SYSTEMTIME,
    pub standard_bias: i32,
    pub daylight_name: [u16; 32],
    pub daylight_date: SYSTEMTIME,
    pub daylight_bias: i32,
}

impl Default for TIME_ZONE_INFORMATION {
    fn default() -> Self {
        Self {
            bias: 0,
            standard_name: [0; 32],
            standard_date: SYSTEMTIME::default(),
            standard_bias: 0,
            daylight_name: [0; 32],
            daylight_date: SYSTEMTIME::default(),
            daylight_bias: 0,
        }
    }
}

/*
 * There are multiple versions of the misc info struct, and each new version includes all
 * fields from the previous versions. We declare them with a macro to avoid repeating
 * the fields excessively.
 */
macro_rules! multi_structs {
    // With no trailing struct left, terminate.
    (@next { $($prev:tt)* }) => {};
    // Declare the next struct, including fields from previous structs.
    (@next { $($prev:tt)* } $(#[$attr:meta])* pub struct $name:ident { $($cur:tt)* } $($tail:tt)* ) => {
        // Prepend fields from previous structs to this struct.
        multi_structs!($(#[$attr])* pub struct $name { $($prev)* $($cur)* } $($tail)*);
    };
    // Declare a single struct.
    ($(#[$attr:meta])* pub struct $name:ident { $( pub $field:ident: $t:tt, )* } $($tail:tt)* ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Pread, SizeWith)]
        pub struct $name {
            $( pub $field: $t, )*
        }
        // Persist its fields down to the following structs.
        multi_structs!(@next { $( pub $field: $t, )* } $($tail)*);
    };
}

multi_structs! {
    /// Miscellaneous process information
    ///
    /// This struct matches the [Microsoft struct][msdn] of the same name.
    ///
    /// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_misc_info
    pub struct MINIDUMP_MISC_INFO {
        pub size_of_info: u32,
        pub flags1: u32,
        pub process_id: u32,
        pub process_create_time: u32,
        pub process_user_time: u32,
        pub process_kernel_time: u32,
    }
    // Includes fields from MINIDUMP_MISC_INFO
    /// Miscellaneous process and system information
    ///
    /// This struct matches the [Microsoft struct][msdn] of the same name.
    ///
    /// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_misc_info_2
    pub struct MINIDUMP_MISC_INFO_2 {
        pub processor_max_mhz: u32,
        pub processor_current_mhz: u32,
        pub processor_mhz_limit: u32,
        pub processor_max_idle_state: u32,
        pub processor_current_idle_state: u32,
    }
    // Includes fields from MINIDUMP_MISC_INFO and MINIDUMP_MISC_INFO_2
    /// Miscellaneous process and system information
    ///
    /// This struct matches the struct of the same name from minidumpapiset.h.
    pub struct MINIDUMP_MISC_INFO_3 {
        pub process_integrity_level: u32,
        pub process_execute_flags: u32,
        pub protected_process: u32,
        pub time_zone_id: u32,
        pub time_zone: TIME_ZONE_INFORMATION,
    }
    // Includes fields from MINIDUMP_MISC_INFO..3
    /// Miscellaneous process and system information
    ///
    /// This struct matches the struct of the same name from minidumpapiset.h.
    pub struct MINIDUMP_MISC_INFO_4 {
        pub build_string: [u16; 260], // MAX_PATH
        pub dbg_bld_str: [u16; 40],
    }

    // Includes fields from MINIDUMP_MISC_INFO..4
    /// Miscellaneous process and system information
    ///
    /// This struct matches the struct of the same name from minidumpapiset.h.
    pub struct MINIDUMP_MISC_INFO_5 {
        pub xstate_data: XSTATE_CONFIG_FEATURE_MSC_INFO,
        pub process_cookie: u32,
    }
    // TODO: read xstate_data and process the extra XSAVE sections at the
    // end of each thread's cpu context.
}

/// A descriptor of the XSAVE context which can be found at the end of
/// each thread's cpu context.
///
/// The sections of this context are dumps of some of the CPUs registers
/// (e.g. one section might contain the contents of the SSE registers).
///
/// Intel documents its XSAVE entries in Volume 1, Chapter 13 of the
/// "Intel 64 and IA-32 Architectures Software Developer’s Manual".
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct XSTATE_CONFIG_FEATURE_MSC_INFO {
    /// The size of this struct.
    pub size_of_info: u32,
    /// The size of the XSAVE context.
    pub context_size: u32,
    /// The bit `enabled_features[i]` indicates that `features[i]` contains valid data.
    pub enabled_features: u64,
    /// The offset and size of each XSAVE entry inside the XSAVE context.
    pub features: [XSTATE_FEATURE; 64],
}

impl Default for XSTATE_CONFIG_FEATURE_MSC_INFO {
    fn default() -> Self {
        Self {
            size_of_info: std::mem::size_of::<XSTATE_CONFIG_FEATURE_MSC_INFO>() as u32,
            context_size: 0,
            enabled_features: 0,
            features: [XSTATE_FEATURE::default(); 64],
        }
    }
}

impl XSTATE_CONFIG_FEATURE_MSC_INFO {
    /// Gets an iterator of all the enabled features.
    pub fn iter(&self) -> XstateFeatureIter {
        XstateFeatureIter { info: self, idx: 0 }
    }
}

/// An iterator of all the enabled features in an XSTATE_CONFIG_FEATURE_MSC_INFO.
#[derive(Debug)]
pub struct XstateFeatureIter<'a> {
    info: &'a XSTATE_CONFIG_FEATURE_MSC_INFO,
    idx: usize,
}

impl<'a> Iterator for XstateFeatureIter<'a> {
    type Item = (usize, XSTATE_FEATURE);
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.info.features.len() {
            let cur_idx = self.idx;
            self.idx += 1;
            if (self.info.enabled_features & (1 << cur_idx)) != 0 {
                return Some((cur_idx, self.info.features[cur_idx]));
            }
        }
        None
    }
}

/// Several known entries in `XSTATE_CONFIG_FEATURE_MSC_INFO.features`.
#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum XstateFeatureIndex {
    LEGACY_FLOATING_POINT = 0,
    LEGACY_SSE = 1,
    GSSE_AND_AVX = 2,
    MPX_BNDREGS = 3,
    MPX_BNDCSR = 4,
    AVX512_KMASK = 5,
    AVX512_ZMM_H = 6,
    ACK512_ZMM = 7,
    XSTATE_IPT = 8,
    XSTATE_LWP = 62,
}

impl XstateFeatureIndex {
    pub fn from_index(idx: usize) -> Option<Self> {
        use XstateFeatureIndex::*;
        match idx {
            0 => Some(LEGACY_FLOATING_POINT),
            1 => Some(LEGACY_SSE),
            2 => Some(GSSE_AND_AVX),
            3 => Some(MPX_BNDREGS),
            4 => Some(MPX_BNDCSR),
            5 => Some(AVX512_KMASK),
            6 => Some(AVX512_ZMM_H),
            7 => Some(ACK512_ZMM),
            8 => Some(XSTATE_IPT),
            62 => Some(XSTATE_LWP),
            _ => None,
        }
    }
}

/// The offset and size of each XSAVE entry inside the XSAVE context.
#[derive(Clone, Copy, Debug, Default, Pread, SizeWith, PartialEq, Eq)]
pub struct XSTATE_FEATURE {
    /// This entry's offset from the start of the context (in bytes).
    pub offset: u32,
    /// This entry's size (in bytes).
    pub size: u32,
}

// For whatever reason Pread array derives use 0u8.into() instead of Default to
// create an initial array to write into. Weird.
impl From<u8> for XSTATE_FEATURE {
    fn from(_input: u8) -> Self {
        XSTATE_FEATURE { offset: 0, size: 0 }
    }
}

bitflags! {
    /// Known flags for `MINIDUMP_MISC_INFO*.flags1`
    pub struct MiscInfoFlags: u32 {
        const MINIDUMP_MISC1_PROCESS_ID            = 0x00000001;
        const MINIDUMP_MISC1_PROCESS_TIMES         = 0x00000002;
        const MINIDUMP_MISC1_PROCESSOR_POWER_INFO  = 0x00000004;
        const MINIDUMP_MISC3_PROCESS_INTEGRITY     = 0x00000010;
        const MINIDUMP_MISC3_PROCESS_EXECUTE_FLAGS = 0x00000020;
        const MINIDUMP_MISC3_TIMEZONE              = 0x00000040;
        const MINIDUMP_MISC3_PROTECTED_PROCESS     = 0x00000080;
        const MINIDUMP_MISC4_BUILDSTRING           = 0x00000100;
        const MINIDUMP_MISC5_PROCESS_COOKIE        = 0x00000200;
    }
}

/// A list of memory regions in a minidump
///
/// This is the format of the [`MINIDUMP_STREAM_TYPE::MemoryInfoListStream`]. The individual
/// [`MINIDUMP_MEMORY_INFO`] entries follow this header in the stream.
///
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_memory_info_list
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct MINIDUMP_MEMORY_INFO_LIST {
    /// The size of this header
    pub size_of_header: u32,
    /// The size of each entry in the list
    pub size_of_entry: u32,
    /// The number of entries in the list
    pub number_of_entries: u64,
}

/// Information about a memory region in a minidump
///
/// This struct matches the [Microsoft struct][msdn] of the same name.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/api/minidumpapiset/ns-minidumpapiset-_minidump_memory_info
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct MINIDUMP_MEMORY_INFO {
    /// The base address of the region of pages
    pub base_address: u64,
    /// The base address of a range of pages in this region
    pub allocation_base: u64,
    /// The memory protection when the region was initially allocated
    ///
    /// See [`MemoryProtection`] for valid values.
    pub allocation_protection: u32,
    pub __alignment1: u32,
    /// The size of the region in which all pages have identical attributes, in bytes
    pub region_size: u64,
    /// The state of the pages in the region
    ///
    /// See [`MemoryState`] for valid values.
    pub state: u32,
    /// The access protection of the pages in the region
    ///
    /// See [`MemoryProtection`] for valid values.
    pub protection: u32,
    /// The type of pages in the region
    ///
    /// See [`MemoryType`] for valid values.
    pub _type: u32,
    pub __alignment2: u32,
}

bitflags! {
    /// Potential values for [`MINIDUMP_MEMORY_INFO::state`]
    pub struct MemoryState: u32 {
        const MEM_COMMIT  = 0x01000;
        const MEM_FREE    = 0x10000;
        const MEM_RESERVE = 0x02000;
    }
}

bitflags! {
    /// Potential values for [`MINIDUMP_MEMORY_INFO::protection`] and `allocation_protection`
    ///
    /// See [Microsoft's documentation][msdn] for details.
    ///
    /// [msdn]: https://docs.microsoft.com/en-us/windows/desktop/Memory/memory-protection-constants
    pub struct MemoryProtection: u32 {
        const PAGE_NOACCESS           = 0x01;
        const PAGE_READONLY           = 0x02;
        const PAGE_READWRITE          = 0x04;
        const PAGE_WRITECOPY          = 0x08;
        const PAGE_EXECUTE            = 0x10;
        const PAGE_EXECUTE_READ       = 0x20;
        const PAGE_EXECUTE_READWRITE  = 0x40;
        const PAGE_EXECUTE_WRITECOPY  = 0x80;
        const ACCESS_MASK             = 0xff;
        const PAGE_GUARD              = 0x100;
        const PAGE_NOCACHE            = 0x200;
        const PAGE_WRITECOMBINE       = 0x400;
    }
}

bitflags! {
    /// Potential values for [`MINIDUMP_MEMORY_INFO::_type`]
    pub struct MemoryType: u32 {
        const MEM_PRIVATE = 0x00020000;
        const MEM_MAPPED  = 0x00040000;
        const MEM_IMAGE   = 0x01000000;
    }
}

/// A Breakpad extension containing some additional process information
///
/// Taken from the definition in Breakpad's [minidump_format.h][fmt].
///
/// [fmt]: https://chromium.googlesource.com/breakpad/breakpad/+/88d8114fda3e4a7292654bd6ac0c34d6c88a8121/src/google_breakpad/common/minidump_format.h#962
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct MINIDUMP_BREAKPAD_INFO {
    pub validity: u32,
    /// The Thread ID of the handler thread
    pub dump_thread_id: u32,
    /// The Thread ID of the thread that requested the dump
    pub requesting_thread_id: u32,
}

bitflags! {
    /// Potential values for [`MINIDUMP_BREAKPAD_INFO::validity`]
    ///
    /// Taken from definitions in Breakpad's [minidump_format.h][fmt].
    ///
    /// [fmt]: https://chromium.googlesource.com/breakpad/breakpad/+/88d8114fda3e4a7292654bd6ac0c34d6c88a8121/src/google_breakpad/common/minidump_format.h#989
    pub struct BreakpadInfoValid: u32 {
        const DumpThreadId       = 1 << 0;
        const RequestingThreadId = 1 << 1;
    }
}

/// A Breakpad extension containing information about an assertion that terminated the process
///
/// Taken from the definition in Breakpad's [minidump_format.h][fmt].
///
/// [fmt]: https://chromium.googlesource.com/breakpad/breakpad/+/88d8114fda3e4a7292654bd6ac0c34d6c88a8121/src/google_breakpad/common/minidump_format.h#998
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct MINIDUMP_ASSERTION_INFO {
    /// The assertion that failed, as a 0-terminated UTF16-LE string
    pub expression: [u16; 128],
    /// The function containing the assertion, as a 0-terminated UTF16-LE string
    pub function: [u16; 128],
    /// The source file containing the assertion, as a 0-terminated UTF16-LE string
    pub file: [u16; 128],
    /// The line number in [`file`] containing the assertion
    pub line: u32,
    /// The assertion type
    pub _type: u32,
}

/// Known values of [`MINIDUMP_ASSERTION_INFO::_type`]
/// Taken from the definition in Breakpad's [minidump_format.h][fmt].
///
/// [fmt]: https://chromium.googlesource.com/breakpad/breakpad/+/88d8114fda3e4a7292654bd6ac0c34d6c88a8121/src/google_breakpad/common/minidump_format.h#1011
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Debug, Primitive)]
pub enum AssertionType {
    Unknown = 0,
    InvalidParameter = 1,
    PureVirtualCall = 2,
}

/// Dynamic linker information for a shared library on 32-bit Linux
///
/// This is functionally equivalent to the data in `struct link_map` defined in <link.h>.
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct LINK_MAP_32 {
    pub addr: u32,
    /// The offset of a string containing the filename of this shared library
    pub name: RVA,
    pub ld: u32,
}

/// DSO debug data for 32-bit Linux minidumps
///
/// Used when converting minidumps to coredumps. This is functionally equivalent to the data
/// in `struct r_debug` defined in <link.h>.
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct DSO_DEBUG_32 {
    /// The version number of this protocol, from `r_debug.r_version`
    pub version: u32,
    /// The offset of an array of [`LINK_MAP_32`] structs
    pub map: RVA,
    /// The number of [`LINK_MAP_32`] entries pointed to by `map`
    pub dso_count: u32,
    /// The address of a function internal to the run-time linker used by debuggers to
    /// set a breakpoint.
    pub brk: u32,
    /// Base address the linker is loaded at
    pub ldbase: u32,
    /// The address of the "dynamic structure"
    pub dynamic: u32,
}

/// Dynamic linker information for a shared library on 64-bit Linux
///
/// This is functionally equivalent to the data in `struct link_map` defined in <link.h>.
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct LINK_MAP_64 {
    pub addr: u64,
    /// The offset of a string containing the filename of this shared library
    pub name: RVA,
    pub ld: u64,
}

/// DSO debug data for 64-bit Linux minidumps
///
/// Used when converting minidumps to coredumps. This is functionally equivalent to the data
/// in `struct r_debug` defined in <link.h>.
#[derive(Debug, Clone, Pread, SizeWith)]
pub struct DSO_DEBUG_64 {
    /// The version number of this protocol, from `r_debug.r_version`
    pub version: u32,
    /// The offset of an array of [`LINK_MAP_64`] structs
    pub map: RVA,
    /// The number of [`LINK_MAP_64`] entries pointed to by `map`
    pub dso_count: u32,
    /// The address of a function internal to the run-time linker used by debuggers to
    /// set a breakpoint.
    pub brk: u64,
    /// Base address the linker is loaded at
    pub ldbase: u64,
    /// The address of the "dynamic structure"
    pub dynamic: u64,
}

/// A variable-length UTF-8-encoded string carried within a minidump file.
///
/// See <https://crashpad.chromium.org/doxygen/structcrashpad_1_1MinidumpUTF8String.html>
#[derive(Debug, Clone)]
pub struct MINIDUMP_UTF8_STRING {
    /// The length of the #Buffer field in bytes, not including the `NUL` terminator.
    ///
    /// This field is interpreted as a byte count, not a count of Unicode code points.
    pub length: u32,
    /// The string, encoded in UTF-8, and terminated with a `NUL` byte.
    pub buffer: Vec<u8>,
}

impl<'a> scroll::ctx::TryFromCtx<'a, Endian> for MINIDUMP_UTF8_STRING {
    type Error = scroll::Error;

    fn try_from_ctx(src: &[u8], endian: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let length: u32 = src.gread_with(offset, endian)?;
        let data: &[u8] = src.gread_with(offset, length as usize + 1)?; // +1 for NUL

        if !data.ends_with(&[0]) {
            return Err(scroll::Error::Custom(
                "Minidump String does not end with NUL byte".to_owned(),
            ));
        }

        let buffer = data.to_vec();
        Ok((Self { length, buffer }, *offset))
    }
}

/// A key-value pair.
///
/// See <https://crashpad.chromium.org/doxygen/structcrashpad_1_1MinidumpSimpleStringDictionaryEntry.html>
#[derive(Clone, Debug, Pread, SizeWith)]
pub struct MINIDUMP_SIMPLE_STRING_DICTIONARY_ENTRY {
    /// RVA of a MinidumpUTF8String containing the key of a key-value pair.
    pub key: RVA,
    /// RVA of a MinidumpUTF8String containing the value of a key-value pair.
    pub value: RVA,
}

/// A list of key-value pairs.
///
/// See <https://crashpad.chromium.org/doxygen/structcrashpad_1_1MinidumpSimpleStringDictionary.html>
#[derive(Clone, Debug, Pread)]
pub struct MINIDUMP_SIMPLE_STRING_DICTIONARY {
    /// The number of key-value pairs present.
    pub count: u32,
}

/// A list of RVA pointers.
///
/// See <https://crashpad.chromium.org/doxygen/structcrashpad_1_1MinidumpRVAList.html>
#[derive(Clone, Debug, Pread)]
pub struct MINIDUMP_RVA_LIST {
    /// The number of pointers present.
    pub count: u32,
}

/// A typed annotation object.
///
/// See <https://crashpad.chromium.org/doxygen/structcrashpad_1_1MinidumpAnnotation.html>
#[derive(Clone, Debug, Pread)]
pub struct MINIDUMP_ANNOTATION {
    /// RVA of a MinidumpUTF8String containing the name of the annotation.
    pub name: RVA,
    /// The type of data stored in the `value` of the annotation. This may correspond to an \a
    /// `MINIDUMP_ANNOTATION_TYPE` or it may be user-defined.
    pub ty: u16,
    /// This field is always `0`.
    pub _reserved: u16,
    /// RVA of a `MinidumpByteArray` to the data for the annotation.
    pub value: RVA,
}

impl MINIDUMP_ANNOTATION {
    /// An invalid annotation. Reserved for internal use.
    ///
    /// See <https://crashpad.chromium.org/doxygen/classcrashpad_1_1Annotation.html#a734ee64cd20afdb78acb8656ed867d34>
    pub const TYPE_INVALID: u16 = 0;
    /// A `NUL`-terminated C-string.
    ///
    /// See <https://crashpad.chromium.org/doxygen/classcrashpad_1_1Annotation.html#a734ee64cd20afdb78acb8656ed867d34>
    pub const TYPE_STRING: u16 = 1;
    /// Clients may declare their own custom types by using values greater than this.
    ///
    /// See <https://crashpad.chromium.org/doxygen/classcrashpad_1_1Annotation.html#a734ee64cd20afdb78acb8656ed867d34>
    pub const TYPE_USER_DEFINED: u16 = 0x8000;
}

/// Additional Crashpad-specific information about a module carried within a minidump file.
///
/// This structure augments the information provided by MINIDUMP_MODULE. The minidump file must
/// contain a module list stream (::kMinidumpStreamTypeModuleList) in order for this structure to
/// appear.
///
/// This structure is versioned. When changing this structure, leave the existing structure intact
/// so that earlier parsers will be able to understand the fields they are aware of, and make
/// additions at the end of the structure. Revise #kVersion and document each field’s validity based
/// on #version, so that newer parsers will be able to determine whether the added fields are valid
/// or not.
///
/// See <https://crashpad.chromium.org/doxygen/structcrashpad_1_1MinidumpModuleCrashpadInfo.html>
#[derive(Clone, Debug, Pread)]
pub struct MINIDUMP_MODULE_CRASHPAD_INFO {
    /// The structure’s version number.
    ///
    /// Readers can use this field to determine which other fields in the structure are valid. Upon
    /// encountering a value greater than `VERSION`, a reader should assume that the structure’s
    /// layout is compatible with the structure defined as having value #kVersion.
    ///
    /// Writers may produce values less than `VERSION` in this field if there is no need for any
    /// fields present in later versions.
    pub version: u32,
    /// A `MinidumpRVAList` pointing to MinidumpUTF8String objects. The module controls the data
    /// that appears here.
    ///
    /// These strings correspond to `ModuleSnapshot::AnnotationsVector()` and do not duplicate
    /// anything in `simple_annotations` or `annotation_objects`.
    pub list_annotations: MINIDUMP_LOCATION_DESCRIPTOR,
    /// A `MinidumpSimpleStringDictionary` pointing to strings interpreted as key-value pairs. The
    /// module controls the data that appears here.
    ///
    /// These key-value pairs correspond to `ModuleSnapshot::AnnotationsSimpleMap()` and do not
    /// duplicate anything in `list_annotations` or `annotation_objects`.
    pub simple_annotations: MINIDUMP_LOCATION_DESCRIPTOR,
    /// A `MinidumpAnnotationList` object containing the annotation objects stored within the
    /// module. The module controls the data that appears here.
    ///
    /// These key-value pairs correspond to `ModuleSnapshot::AnnotationObjects()` and do not
    /// duplicate anything in `list_annotations` or `simple_annotations`.
    pub annotation_objects: MINIDUMP_LOCATION_DESCRIPTOR,
}

impl MINIDUMP_MODULE_CRASHPAD_INFO {
    /// The structure’s version number.
    ///
    /// Readers can use this field to determine which other fields in the structure are valid. Upon
    /// encountering a value greater than `VERSION`, a reader should assume that the structure’s
    /// layout is compatible with the structure defined as having value #kVersion.
    ///
    /// Writers may produce values less than `VERSION` in this field if there is no need for any
    /// fields present in later versions.
    pub const VERSION: u32 = 1;
}

/// A link between a `MINIDUMP_MODULE` structure and additional Crashpad-specific information about a
/// module carried within a minidump file.
///
/// See <https://crashpad.chromium.org/doxygen/structcrashpad_1_1MinidumpModuleCrashpadInfoLink.html>
#[derive(Clone, Debug, Pread)]
pub struct MINIDUMP_MODULE_CRASHPAD_INFO_LINK {
    /// A link to a MINIDUMP_MODULE structure in the module list stream.
    ///
    /// This field is an index into `MINIDUMP_MODULE_LIST::Modules`. This field’s value must be in
    /// the range of `MINIDUMP_MODULE_LIST::NumberOfEntries`.
    pub minidump_module_list_index: u32,

    /// A link to a MinidumpModuleCrashpadInfo structure.
    ///
    /// MinidumpModuleCrashpadInfo structures are accessed indirectly through
    /// `MINIDUMP_LOCATION_DESCRIPTOR` pointers to allow for future growth of the
    /// `MinidumpModuleCrashpadInfo` structure.
    pub location: MINIDUMP_LOCATION_DESCRIPTOR,
}

/// Additional Crashpad-specific information about modules carried within a minidump file.
///
/// This structure augments the information provided by `MINIDUMP_MODULE_LIST`. The minidump file
/// must contain a module list stream (::kMinidumpStreamTypeModuleList) in order for this structure
/// to appear.
///
/// `MinidumpModuleCrashpadInfoList::count` may be less than the value of
/// `MINIDUMP_MODULE_LIST::NumberOfModules` because not every `MINIDUMP_MODULE` structure carried
/// within the minidump file will necessarily have Crashpad-specific information provided by a
/// `MinidumpModuleCrashpadInfo` structure.
///
/// See <https://crashpad.chromium.org/doxygen/structcrashpad_1_1MinidumpModuleCrashpadInfoList.html>
#[derive(Clone, Debug, Pread)]
pub struct MINIDUMP_MODULE_CRASHPAD_INFO_LIST {
    /// The number of key-value pairs present.
    pub count: u32,
}

/// Additional Crashpad-specific information carried within a minidump file.
///
/// This structure is versioned. When changing this structure, leave the existing structure intact
/// so that earlier parsers will be able to understand the fields they are aware of, and make
/// additions at the end of the structure. Revise #kVersion and document each field’s validity based
/// on `version`, so that newer parsers will be able to determine whether the added fields are valid
/// or not.
///
/// See <https://crashpad.chromium.org/doxygen/structcrashpad_1_1MinidumpCrashpadInfo.html>
#[derive(Clone, Debug, Pread, SizeWith)]
pub struct MINIDUMP_CRASHPAD_INFO {
    /// The structure’s version number.
    ///
    /// Readers can use this field to determine which other fields in the structure are valid. Upon
    /// encountering a value greater than `VERSION`, a reader should assume that the structure’s
    /// layout is compatible with the structure defined as having value #kVersion.
    ///
    /// Writers may produce values less than `VERSION` in this field if there is no need for any
    /// fields present in later versions.
    pub version: u32,
    /// A `Uuid` identifying an individual crash report.
    ///
    /// This provides a stable identifier for a crash even as the report is converted to different
    /// formats, provided that all formats support storing a crash report ID.
    ///
    /// If no identifier is available, this field will contain zeroes.
    pub report_id: GUID,
    /// A `Uuid` identifying the client that crashed.
    ///
    /// Client identification is within the scope of the application, but it is expected that the
    /// identifier will be unique for an instance of Crashpad monitoring an application or set of
    /// applications for a user. The identifier shall remain stable over time.
    ///
    /// If no identifier is available, this field will contain zeroes.
    pub client_id: GUID,
    /// A MinidumpSimpleStringDictionary pointing to strings interpreted as key-value pairs.
    ///
    /// These key-value pairs correspond to Crashpad's `ProcessSnapshot::AnnotationsSimpleMap()`.
    pub simple_annotations: MINIDUMP_LOCATION_DESCRIPTOR,
    /// A pointer to a MinidumpModuleCrashpadInfoList structure.
    pub module_list: MINIDUMP_LOCATION_DESCRIPTOR,
}

impl MINIDUMP_CRASHPAD_INFO {
    /// The structure’s currently-defined version number.
    pub const VERSION: u32 = 1;
}

/// MacOS __DATA,__crash_info data.
///
/// This is the format of the [`MINIDUMP_STREAM_TYPE::MozMacosCrashInfoStream`]. The individual
/// [`MINIDUMP_MAC_CRASH_INFO_RECORD`] entries follow this header in the stream.
#[derive(Debug, Pread, SizeWith)]
pub struct MINIDUMP_MAC_CRASH_INFO {
    pub stream_type: u32,
    /// The number of [`MINIDUMP_MAC_CRASH_INFO_RECORD`]s.
    pub record_count: u32,
    /// The size of the "fixed-size" part of MINIDUMP_MAC_CRASH_INFO_RECORD.
    /// Used to offset to the variable-length portion of the struct, where
    /// C-strings are stored. This allows us to access all the fields we know
    /// about, even when newer versions of this format introduce new ones.
    pub record_start_size: u32,
    pub records: [MINIDUMP_LOCATION_DESCRIPTOR; 20],
}

// MozMacosCrashInfoStream is a versioned format where new fields are added to
// the end of the struct, but there are also variable-length c-string fields
// that follow the "fixed-size" fields. As such, the versioned strings are separated
// out into their own separate struct with the same version. So e.g.
//
// MINIDUMP_MAC_CRASH_INFO_RECORD_4 should be paired with MINIDUMP_MAC_CRASH_INFO_RECORD_STRINGS_4

multi_structs! {
    /// Contents of MacOS's `<CrashReporterClient.h>`'s `crashreporter_annotations_t`,
    /// but with the by-reference C-strings hoisted out to the end of the struct
    /// and inlined (so this is a variable-length struct).
    ///
    /// The variable-length strings are listed in [`MINIDUMP_MAC_CRASH_INFO_RECORD_STRINGS`].
    /// Use [`MINIDUMP_MAC_CRASH_INFO::record_start_size`] to access them.
    pub struct MINIDUMP_MAC_CRASH_INFO_RECORD {
      pub stream_type: u64,
      // Version of this format, currently at 5.
      //
      // Although theoretically this field being here means we can support multiple
      // versions of this struct in one [`MINIDUMP_MAC_CRASH_INFO`] stream, our reliance on
      // [`MINIDUMP_MAC_CRASH_INFO::record_start_size`] means we can't actually handle
      // such a heterogeneous situation. So all records should have the same version value.
      pub version: u64,
    }
    // Includes fields from MINIDUMP_MAC_CRASH_INFO_RECORD
    /// Contents of MacOS's `<CrashReporterClient.h>`'s `crashreporter_annotations_t`,
    /// but with the by-reference C-strings hoisted out to the end of the struct
    /// and inlined (so this is a variable-length struct).
    ///
    /// The variable-length strings are listed in [`MINIDUMP_MAC_CRASH_INFO_RECORD_STRINGS_4`].
    /// Use [`MINIDUMP_MAC_CRASH_INFO::record_start_size`] to access them.
    pub struct MINIDUMP_MAC_CRASH_INFO_RECORD_4 {
        pub thread: u64,
        pub dialog_mode: u64,
    }
    // Includes fields from MINIDUMP_MAC_CRASH_INFO_RECORD and MINIDUMP_MAC_CRASH_INFO_RECORD_4
    /// Contents of MacOS's `<CrashReporterClient.h>`'s `crashreporter_annotations_t`,
    /// but with the by-reference C-strings hoisted out to the end of the struct
    /// and inlined (so this is a variable-length struct).
    ///
    /// The variable-length strings are listed in [`MINIDUMP_MAC_CRASH_INFO_RECORD_STRINGS_5`].
    /// Use [`MINIDUMP_MAC_CRASH_INFO::record_start_size`] to access them.
    pub struct MINIDUMP_MAC_CRASH_INFO_RECORD_5 {
        pub abort_cause: u64,
    }
}

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ replace_expr!($tts 1usize))*};
}

// Like multi_structs but specialized for a struct of strings that can be set by index.
macro_rules! multi_strings {
    // With no trailing struct left, terminate.
    (@next { $($prev:tt)* }) => {};
    // Declare the next struct, including fields from previous structs.
    (@next { $($prev:tt)* } $(#[$attr:meta])* pub struct $name:ident { $($cur:tt)* } $($tail:tt)* ) => {
        // Prepend fields from previous structs to this struct.
        multi_strings!($(#[$attr])* pub struct $name { $($prev)* $($cur)* } $($tail)*);
    };
    // Declare a single struct.
    ($(#[$attr:meta])* pub struct $name:ident { $( pub $field:ident: $t:tt, )* } $($tail:tt)* ) => {
        $(#[$attr])*
        #[derive(Default, Debug, Clone)]
        pub struct $name {
            $( pub $field: $t, )*
        }

        impl $name {
            pub fn num_strings() -> usize {
                count_tts!($($t)*)
            }

            #[allow(unused_variables, unused_mut)]
            pub fn set_string(&mut self, idx: usize, string: String) {
                let mut cur_idx = 0;
                $(if cur_idx == idx {
                    self.$field = string;
                    return;
                }
                cur_idx += 1;
                )*
                panic!("string index out of bounds {} >= {}", idx, cur_idx);
            }
        }

        // Persist its fields down to the following structs.
        multi_strings!(@next { $( pub $field: $t, )* } $($tail)*);
    };
}

multi_strings! {
    /// Variable-length data for [`MINIDUMP_MAC_CRASH_INFO_RECORD`].
    pub struct MINIDUMP_MAC_CRASH_INFO_RECORD_STRINGS {
        // No strings in the base version
    }

    // Includes fields from [`MINIDUMP_MAC_CRASH_INFO_RECORD_STRINGS`]
    /// Variable-length data for [`MINIDUMP_MAC_CRASH_INFO_RECORD_4`].
    pub struct MINIDUMP_MAC_CRASH_INFO_RECORD_STRINGS_4 {
        pub module_path: String,
        pub message: String,
        pub signature_string: String,
        pub backtrace: String,
        pub message2: String,
    }

    // Includes fields from [`MINIDUMP_MAC_CRASH_INFO_RECORD_STRINGS_4`]
    /// Variable-length data for [`MINIDUMP_MAC_CRASH_INFO_RECORD_5`].
    pub struct MINIDUMP_MAC_CRASH_INFO_RECORD_STRINGS_5 {
        // No new strings
    }
}

/// The maximum supported size of a C-string in [`MINIDUMP_MAC_CRASH_INFO_RECORD`].
///
/// Assume the stream is corrupted if a string is longer than this.
pub const MAC_CRASH_INFO_STRING_MAX_SIZE: usize = 8192;

/// The maximum supported count of [`MINIDUMP_MAC_CRASH_INFO_RECORD`]s.
///
/// In principle there should only be one or two non-empty __DATA,__crash_info
/// sections per process. But the __crash_info section is almost entirely
/// undocumented, so just in case we set a large maximum.
pub const MAC_CRASH_INFOS_MAX: usize = 20;

bitflags! {
    /// Possible values of [`ARMCpuInfo::elf_hwcaps`]
    ///
    /// This matches the Linux kernel definitions from [<asm/hwcaps.h>][hwcap].
    ///
    /// [hwcap]: https://elixir.bootlin.com/linux/latest/source/arch/arm/include/uapi/asm/hwcap.h
    pub struct ArmElfHwCaps: u32 {
        const HWCAP_SWP       = (1 << 0);
        const HWCAP_HALF      = (1 << 1);
        const HWCAP_THUMB     = (1 << 2);
        const HWCAP_26BIT     = (1 << 3);
        const HWCAP_FAST_MULT = (1 << 4);
        const HWCAP_FPA       = (1 << 5);
        const HWCAP_VFP       = (1 << 6);
        const HWCAP_EDSP      = (1 << 7);
        const HWCAP_JAVA      = (1 << 8);
        const HWCAP_IWMMXT    = (1 << 9);
        const HWCAP_CRUNCH    = (1 << 10);
        const HWCAP_THUMBEE   = (1 << 11);
        const HWCAP_NEON      = (1 << 12);
        const HWCAP_VFPv3     = (1 << 13);
        const HWCAP_VFPv3D16  = (1 << 14);
        const HWCAP_TLS       = (1 << 15);
        const HWCAP_VFPv4     = (1 << 16);
        const HWCAP_IDIVA     = (1 << 17);
        const HWCAP_IDIVT     = (1 << 18);
        const HWCAP_VFPD32    = (1 << 19);
        const HWCAP_IDIV      = ArmElfHwCaps::HWCAP_IDIVA.bits | Self::HWCAP_IDIVT.bits;
        const HWCAP_LPAE      = (1 << 20);
        const HWCAP_EVTSTRM   = (1 << 21);
    }
}
