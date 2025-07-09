# TODO: Missing Breezy Python API Wrappers

This document lists Breezy Python APIs that are not yet wrapped in the breezyshim Rust crate.

## Core Modules

### breezy.annotate
- File annotation/blame functionality
- Shows who last modified each line and when

### breezy.bundle
- Bundle creation and management
- Serialization of changesets for transport

### breezy.check
- Repository and branch consistency checking
- Integrity verification tools

### breezy.filters
- Content filtering framework
- Text conversion filters (e.g., line endings, keywords)

### breezy.foreign
- Foreign VCS mapping infrastructure
- Base classes for foreign VCS support

### breezy.globbing
- Advanced file pattern matching
- Glob and regex pattern support

### breezy.graph
- Graph algorithms for revision DAGs
- Currently only exception is imported, not the actual graph operations

### breezy.hashcache
- File content hash caching
- Performance optimization for file comparisons

### breezy.help / breezy.help_topics
- Help system infrastructure
- Command and topic documentation

### breezy.hooks
- Hook registration and execution framework
- Currently has limited usage in the codebase

### breezy.info
- Repository/branch/tree information display
- Statistics and metadata reporting

### breezy.inventory
- File inventory management
- Tree content tracking

### breezy.lazy_import
- Lazy module importing infrastructure
- Performance optimization

### breezy.lru_cache
- Least Recently Used cache implementation
- Memory management utilities

### breezy.mail_client
- Email client integration
- Sending patches/merge proposals via email

### breezy.memorytree
- In-memory tree implementation
- Testing and temporary tree operations

### breezy.missing
- Missing revision detection
- Branch divergence analysis

### breezy.reconcile
- Repository reconciliation
- Data consistency repair

### breezy.reconfigure
- Branch/repository reconfiguration
- Format conversion utilities

### breezy.registry
- Plugin and component registry
- Dynamic registration system

### breezy.remote
- Remote repository/branch operations
- Smart server protocol

### breezy.rio
- RIO (Recursive, Indented, Ordered) format
- Structured data serialization

### breezy.rules
- File handling rules
- Pattern-based file treatment

### breezy.send
- Bundle sending functionality
- Email/file export of changes

### breezy.shelf
- Shelving/unshelving changes
- Temporary change storage

### breezy.smtp_connection
- SMTP connection handling
- Email infrastructure

### breezy.split
- Repository splitting
- Partial repository extraction

### breezy.switch
- Branch switching
- Lightweight checkouts

### breezy.tag
- Tag management operations
- Currently has limited wrapper coverage

### breezy.testament
- Testament generation
- Cryptographic summary of tree state

### breezy.textfile
- Text file utilities
- Line ending detection/conversion

### breezy.textmerge
- Text merging algorithms
- Three-way merge implementation

### breezy.trace
- Logging and debugging infrastructure
- Error reporting utilities

### breezy.tsort
- Topological sorting
- Dependency ordering

### breezy.uncommit
- Uncommit functionality
- Revision removal from branch

### breezy.upgrade
- Repository/branch format upgrades
- Migration utilities

### breezy.version
- Version information
- Build/runtime version details

### breezy.views
- Filtered working tree views
- Partial checkout support

### breezy.weave
- Weave merge algorithm
- Historical VCS algorithm

### breezy.xml_serializer
- XML serialization
- Legacy format support

## Platform-Specific

### breezy.win32utils
- Windows-specific utilities
- Platform compatibility layer

### breezy.walkdirs
- Optimized directory traversal
- OS-specific implementations

## Storage Formats

### breezy.chunk_writer
- Chunked data writing
- Storage optimization

### breezy.groupcompress
- Group compression format
- Storage backend

### breezy.knit
- Knit storage format
- Historical storage backend

### breezy.multiparent
- Multi-parent diff format
- Storage optimization

### breezy.pack
- Pack file format
- Repository storage

### breezy.vf_repository
- Versioned file repository
- Storage abstraction

### breezy.vf_search
- Versioned file searching
- Repository queries

## Utilities

### breezy.bugtracker
- Bug tracker integration
- URL generation for bug references

### breezy.directory_service
- Directory service lookups
- Special URL handling

### breezy.lsprof
- Profiling support
- Performance analysis

### breezy.tuned_gzip
- Optimized gzip implementation
- Compression utilities

## Known TODOs from Code Comments

1. **Large error enum variants need boxing** (lib.rs)
2. **InterGitRepository methods placement** (interrepository.rs)
3. **Stat conversion from Python to Rust metadata** (tree.rs)
4. **ForeignRepository methods placement** (repository.rs)
5. **Workspace file reset optimization** (workspace.rs)
6. **ChangeLogError implementation** (debian/error.rs)

## Implementation Priority

High priority (core functionality):
- breezy.annotate
- breezy.tag (complete implementation)
- breezy.hooks (complete implementation)
- breezy.graph (complete implementation)
- breezy.info

Medium priority (common operations):
- breezy.bundle
- breezy.check
- breezy.filters
- breezy.missing
- breezy.shelf
- breezy.switch
- breezy.uncommit

Low priority (specialized or legacy):
- breezy.weave
- breezy.knit
- breezy.xml_serializer
- Storage format modules
- Platform-specific modules

## Notes

- Some modules may not need wrapping if they're internal implementation details
- Priority should be given to user-facing functionality
- Consider whether some Python modules can be reimplemented in pure Rust instead of wrapped