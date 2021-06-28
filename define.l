.TH DEFINE 1
.SH NAME
define \- store and lookup terms in a filesystem based dictionary
.SH SYNOPSIS
.B define
[\fB\-c\fR | \fB\-\-caseful\fR] [\fB\-V\fR | \fB\-\-version\fR] [\fB\-h\fR | \fB\-\-help\fR] [\fB\-l\fR \f\Ilevel\f\R | \fB\-\-logs\fR \f\Ilevel\f\R]
.IR key
.IR definition
.SH DESCRIPTION
.B define
is simple tool for curation and lookup of definitions and for other dictionary-like purposes.
.SH OPTIONS
.TP
.BR [\fB\-c\fR\ |\ \fB\-\-caseful\fR]
Force the use of the exact case of the key for creating or reading the dictionary entry. By default lower-case versions will be used regardless of what case the key is provided in.
.TP
.BR [\fB\-V\fR\ |\ \fB\-\-version\fR]
Output the version of \fBdefine\fR being executed
.TP
.BR [\fB\-h\fR\ |\ \fB\-\-help\fR]
Output basic usage information (an abbreviated version of this man page)
.TP
.BR [\fB\-l\fR\ \fIlevel\fR\ |\ \fB\-\-logs\fR\ \fIlevel\fR]
Ouput additional debugging information. The \fIlevel\fR supplied can be one of \fBtrace\fR, \fBdebug\fR, \fBinfo\fR, \fBwarn\fR, or \fBerror\fR to provide increasing degrees of detail.
.SH FILES
Unless overridden by the \fBDEFINITIONS_DICTIONARY_PATH\fR environment variable definitions will be stored and looked up from the following directories
.PP
.RE
.I \fB*\fR ~/.config/define
.br
.I \fB*\fR ~/.define
.br
.I \fB*\fR /etc/define
.RS

.SH EXAMPLES
To store a term in the dictionary provide both the key and the phrase
.PP
.nf
.RS
.B $ define LOTR """Lord of the Rings"""
.PP
Lord of the Rings
.RE
.fi
.PP
To lookup a term in the dictionary provide just the key
.PP
.nf
.RS
.B $ define LOTR
.PP
Lord of the Rings
.RE
.fi
.PP
Additional definitions can be given and will appear on subsequent lines
.PP
.nf
.RS
.B $ define FAQ """Frequently Asked Questions"""
.PP
Frequently Asked Questions
.PP
.B $ define FAQ """Food And Quarks"""
.PP
Frequently Asked Questions
Food And Quarks
.PP
.B $ define FAQ
.PP
Frequently Asked Questions
Food And Quarks
.SH NOTES
Quotation marks are usually required around phrases (and occasionally around keys) to avoid shell globbing issues.
.SH TODO
The author intends to carry out the following changes or additions to the tool
.PP
.RE
 \fB*\fR Normalisation of entered phrases (whitespace removal etc.)
.br
 \fB*\fR When storing, ignore definitions that already exist on some line in the corresponding dictionary file
.RS
