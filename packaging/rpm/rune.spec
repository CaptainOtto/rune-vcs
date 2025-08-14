Name:           rune
Version:        0.0.2
Release:        1%{?dist}
Summary:        Rune - A modern, intelligent version control system
License:        Apache-2.0
URL:            https://github.com/CaptainOtto/rune-vcs
Source0:        rune-%{version}.tar.gz

BuildRequires:  gcc
Requires:       glibc

%description
Rune is a next-generation distributed version control system designed for
performance, scalability, and ease of use. It features intelligent file
analysis, advanced branching capabilities, and seamless integration with
modern development workflows.

Key features:
* Fast and efficient operations
* Intelligent file tracking and analysis
* Advanced branching and merging
* Built-in large file support (LFS)
* Modern CLI with enhanced user experience
* Cross-platform compatibility

%prep
%setup -q

%build
# Binary is pre-built

%install
rm -rf $RPM_BUILD_ROOT
mkdir -p $RPM_BUILD_ROOT/usr/bin
mkdir -p $RPM_BUILD_ROOT/usr/share/doc/rune
mkdir -p $RPM_BUILD_ROOT/usr/share/man/man1

install -m 755 rune $RPM_BUILD_ROOT/usr/bin/
install -m 644 README.md $RPM_BUILD_ROOT/usr/share/doc/rune/
install -m 644 docs/*.md $RPM_BUILD_ROOT/usr/share/doc/rune/ || true

# Create and install man page
cat > rune.1 << 'EOF'
.TH RUNE 1 "August 2025" "rune 0.0.2" "User Commands"
.SH NAME
rune \- modern distributed version control system
.SH SYNOPSIS
.B rune
[\fIOPTIONS\fR] \fICOMMAND\fR [\fIARGS\fR]
.SH DESCRIPTION
Rune is a modern, intelligent distributed version control system designed for performance and ease of use.
.SH OPTIONS
.TP
\fB\-v\fR, \fB\-\-verbose\fR
Enable verbose output
.TP
\fB\-q\fR, \fB\-\-quiet\fR
Suppress non-essential output
.TP
\fB\-y\fR, \fB\-\-yes\fR
Assume yes for confirmation prompts
.SH COMMANDS
.TP
\fBinit\fR
Initialize a new repository
.TP
\fBadd\fR \fIfiles\fR
Add files to staging area
.TP
\fBcommit\fR \fB\-m\fR \fImessage\fR
Commit staged changes
.TP
\fBstatus\fR
Show working directory status
.TP
\fBlog\fR
Show commit history
.SH AUTHORS
Rune Maintainers <dev@example.invalid>
.SH SEE ALSO
git(1)
EOF

gzip -c rune.1 > $RPM_BUILD_ROOT/usr/share/man/man1/rune.1.gz

%files
%defattr(-,root,root,-)
/usr/bin/rune
/usr/share/doc/rune/
/usr/share/man/man1/rune.1.gz

%changelog
* Wed Aug 14 2025 Rune Maintainers <dev@example.invalid> - 0.0.2-1
- Release 0.0.2: Complete Phase 2 with advanced VCS operations and UX enhancements
- Added verbose/quiet modes with --verbose and --quiet flags
- Enhanced error messages with helpful suggestions
- Implemented progress bars for long operations
- Added confirmation prompts for destructive operations
- Completed clone, fetch, pull, push commands with local repository support
- Improved user experience with context-aware styling and feedback

* Mon Aug 12 2025 Rune Maintainers <dev@example.invalid> - 0.0.1-1
- Initial release with core VCS functionality
- Basic repository operations: init, add, commit, status, log
- Branch management: create, list, checkout, merge
- Advanced operations: diff, reset, show
- Professional CLI with colorized output
- Cross-platform support and comprehensive testing
