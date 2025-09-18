# Chapter 16: File I/O & Serialization

## Learning Objectives
- Master file operations with proper error handling and buffering strategies
- Work with path manipulation and filesystem operations safely
- Use serde for JSON, TOML, and binary serialization/deserialization
- Build robust CLI applications with clap argument parsing
- Understand async vs sync I/O trade-offs and when to use each
- Handle cross-platform file system differences effectively

## File Operations and Error Handling

Rust provides comprehensive file I/O capabilities through `std::fs` and `std::io` modules:

```rust
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

// Basic file reading with proper error handling
fn read_file_to_string(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// More efficient for large files: buffered reading
fn read_file_buffered(path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let lines: Result<Vec<_>, _> = reader.lines().collect();
    lines
}

// Writing files with different options
fn write_file_examples(path: &Path, content: &str) -> io::Result<()> {
    // Simple write (overwrites existing file)
    std::fs::write(path, content)?;
    
    // More control with OpenOptions
    let mut file = OpenOptions::new()
        .create(true)          // Create if doesn't exist
        .write(true)           // Allow writing
        .append(true)          // Append to existing content
        .open(path)?;
    
    writeln!(file, "Additional line: {}", content)?;
    
    // Ensure data is written to disk
    file.flush()?;
    
    Ok(())
}

// Buffered writing for better performance
fn write_file_buffered(path: &Path, lines: &[String]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    
    for line in lines {
        writeln!(writer, "{}", line)?;
    }
    
    // Important: flush buffer before dropping
    writer.flush()?;
    Ok(())
}

// Copy files with progress tracking
fn copy_file_with_progress<P: AsRef<Path>>(
    source: P,
    dest: P,
    buffer_size: usize,
) -> io::Result<u64> {
    let mut source_file = File::open(&source)?;
    let mut dest_file = File::create(&dest)?;
    
    let mut buffer = vec![0; buffer_size];
    let mut total_copied = 0u64;
    
    loop {
        let bytes_read = source_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break; // EOF reached
        }
        
        dest_file.write_all(&buffer[..bytes_read])?;
        total_copied += bytes_read as u64;
        
        // Progress callback could go here
        if total_copied % (1024 * 1024) == 0 { // Every MB
            println!("Copied {} MB", total_copied / (1024 * 1024));
        }
    }
    
    dest_file.flush()?;
    Ok(total_copied)
}
```

**C++/C# Comparison:**
- **C++**: std::ifstream/ofstream, manual resource management
- **C#**: FileStream, StreamReader/Writer with using statements
- **Rust**: Built-in RAII, explicit error handling, zero-cost abstractions

## Path Manipulation and Filesystem Operations

Working safely with paths across platforms:

```rust
use std::path::{Path, PathBuf};
use std::fs;
use std::env;

// Path construction and manipulation
fn path_operations() -> io::Result<()> {
    // Get current directory
    let current_dir = env::current_dir()?;
    println!("Current directory: {}", current_dir.display());
    
    // Build paths safely
    let mut config_path = current_dir.clone();
    config_path.push("config");
    config_path.push("app.toml");
    
    // Alternative: using join
    let data_path = current_dir.join("data").join("users.json");
    
    // Path components
    if let Some(parent) = config_path.parent() {
        println!("Config directory: {}", parent.display());
    }
    
    if let Some(filename) = config_path.file_name() {
        println!("Config filename: {:?}", filename);
    }
    
    if let Some(extension) = config_path.extension() {
        println!("File extension: {:?}", extension);
    }
    
    // Path validation
    if config_path.exists() {
        println!("Config file exists");
        
        if config_path.is_file() {
            println!("It's a file");
        }
        
        let metadata = fs::metadata(&config_path)?;
        println!("File size: {} bytes", metadata.len());
        println!("Modified: {:?}", metadata.modified()?);
    }
    
    Ok(())
}

// Directory operations
fn directory_operations() -> io::Result<()> {
    let base_dir = Path::new("./workspace");
    
    // Create directories
    fs::create_dir_all(base_dir.join("logs"))?;
    fs::create_dir_all(base_dir.join("data/cache"))?;
    
    // List directory contents
    for entry in fs::read_dir(base_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            println!("File: {}", path.display());
        } else if path.is_dir() {
            println!("Directory: {}", path.display());
        }
    }
    
    // Recursive directory traversal
    fn visit_dir(dir: &Path, depth: usize) -> io::Result<()> {
        if depth > 5 { // Prevent infinite recursion
            return Ok(());
        }
        
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            println!("{:indent$}{}", "", path.file_name().unwrap().to_string_lossy(),
                    indent = depth * 2);
            
            if path.is_dir() {
                visit_dir(&path, depth + 1)?;
            }
        }
        Ok(())
    }
    
    visit_dir(base_dir, 0)?;
    
    Ok(())
}

// File system operations with error handling
fn filesystem_operations() -> io::Result<()> {
    let temp_file = Path::new("temp.txt");
    let backup_file = Path::new("temp.txt.backup");
    
    // Create a temporary file
    fs::write(temp_file, "temporary content")?;
    
    // Copy file
    fs::copy(temp_file, backup_file)?;
    
    // Move/rename file
    let renamed_file = Path::new("renamed.txt");
    fs::rename(backup_file, renamed_file)?;
    
    // Remove files
    fs::remove_file(temp_file)?;
    fs::remove_file(renamed_file)?;
    
    // Create and remove directories
    let test_dir = Path::new("test_directory");
    fs::create_dir(test_dir)?;
    fs::remove_dir(test_dir)?;
    
    Ok(())
}

// Cross-platform path handling
fn cross_platform_paths() {
    // Platform-specific separators
    println!("Path separator: {}", std::path::MAIN_SEPARATOR);
    
    // Build paths that work on all platforms
    let home_dir = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))  // Windows fallback
        .expect("Could not find home directory");
    
    let config_path = Path::new(&home_dir).join(".config").join("myapp");
    println!("Config path: {}", config_path.display());
    
    // Handle different path formats
    let path_str = if cfg!(windows) {
        r"C:\Users\John\Documents\file.txt"
    } else {
        "/home/john/Documents/file.txt"
    };
    
    let path = Path::new(path_str);
    println!("Parsed path: {}", path.display());
}
```

## Serialization with Serde

Serde provides powerful serialization/deserialization capabilities:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Basic serializable structures
#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: u32,
    username: String,
    email: String,
    active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile_picture: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    database_url: String,
    port: u16,
    debug: bool,
    features: Vec<String>,
    cache_settings: CacheConfig,
    #[serde(default)]
    optional_settings: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CacheConfig {
    ttl_seconds: u64,
    max_size: usize,
    #[serde(rename = "enabled")]
    is_enabled: bool,
}

// JSON serialization
fn json_examples() -> Result<(), Box<dyn std::error::Error>> {
    let user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        active: true,
        profile_picture: Some("avatar.jpg".to_string()),
    };
    
    // Serialize to JSON string
    let json_string = serde_json::to_string(&user)?;
    println!("JSON: {}", json_string);
    
    // Pretty-print JSON
    let json_pretty = serde_json::to_string_pretty(&user)?;
    println!("Pretty JSON:\n{}", json_pretty);
    
    // Deserialize from JSON
    let user_from_json: User = serde_json::from_str(&json_string)?;
    println!("Deserialized: {:?}", user_from_json);
    
    // Work with JSON files
    let users = vec![user.clone(), User {
        id: 2,
        username: "bob".to_string(),
        email: "bob@example.com".to_string(),
        active: false,
        profile_picture: None,
    }];
    
    // Write to file
    let json_file = File::create("users.json")?;
    serde_json::to_writer_pretty(json_file, &users)?;
    
    // Read from file
    let json_file = File::open("users.json")?;
    let users_from_file: Vec<User> = serde_json::from_reader(json_file)?;
    println!("Users from file: {:#?}", users_from_file);
    
    Ok(())
}

// TOML configuration files
fn toml_examples() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        database_url: "postgresql://localhost:5432/myapp".to_string(),
        port: 8080,
        debug: false,
        features: vec!["auth".to_string(), "api".to_string()],
        cache_settings: CacheConfig {
            ttl_seconds: 3600,
            max_size: 10000,
            is_enabled: true,
        },
        optional_settings: {
            let mut settings = HashMap::new();
            settings.insert("theme".to_string(), "dark".to_string());
            settings.insert("language".to_string(), "en".to_string());
            settings
        },
    };
    
    // Serialize to TOML
    let toml_string = toml::to_string(&config)?;
    println!("TOML configuration:\n{}", toml_string);
    
    // Write to file
    std::fs::write("config.toml", &toml_string)?;
    
    // Read from file
    let toml_content = std::fs::read_to_string("config.toml")?;
    let config_from_toml: Config = toml::from_str(&toml_content)?;
    println!("Config from TOML: {:#?}", config_from_toml);
    
    Ok(())
}

// Binary serialization with bincode
fn binary_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let users = vec![
        User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            active: true,
            profile_picture: Some("avatar.jpg".to_string()),
        },
        User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
            active: false,
            profile_picture: None,
        },
    ];
    
    // Serialize to binary
    let encoded: Vec<u8> = bincode::serialize(&users)?;
    println!("Binary size: {} bytes", encoded.len());
    
    // Write binary data to file
    std::fs::write("users.bin", &encoded)?;
    
    // Read binary data from file
    let binary_data = std::fs::read("users.bin")?;
    let decoded_users: Vec<User> = bincode::deserialize(&binary_data)?;
    
    println!("Decoded users: {:#?}", decoded_users);
    
    // Compare sizes
    let json_size = serde_json::to_string(&users)?.len();
    println!("JSON size: {} bytes, Binary size: {} bytes", json_size, encoded.len());
    
    Ok(())
}

// Custom serialization with serde attributes
#[derive(Serialize, Deserialize, Debug)]
struct CustomSerialization {
    #[serde(rename = "customName")]
    name: String,
    
    #[serde(with = "timestamp_format")]
    created_at: std::time::SystemTime,
    
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    tags: Vec<String>,
    
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

mod timestamp_format {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::time::SystemTime;
    
    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(SystemTime::UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_u64(duration.as_secs())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = u64::deserialize(deserializer)?;
        Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp))
    }
}
```

## Command Line Interfaces with Clap

Building robust CLI applications:

```rust
use clap::{Parser, Subcommand, ArgGroup};
use std::path::PathBuf;

// Main CLI structure
#[derive(Parser, Debug)]
#[command(name = "file-manager")]
#[command(about = "A comprehensive file management tool")]
#[command(version = "1.0.0")]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    
    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Copy files or directories
    Copy {
        /// Source path
        source: PathBuf,
        /// Destination path
        destination: PathBuf,
        /// Copy recursively for directories
        #[arg(short, long)]
        recursive: bool,
        /// Overwrite existing files
        #[arg(short, long)]
        force: bool,
    },
    /// Find files matching criteria
    Find {
        /// Directory to search in
        #[arg(default_value = ".")]
        path: PathBuf,
        /// File name pattern
        #[arg(short, long)]
        name: Option<String>,
        /// File extension filter
        #[arg(short, long)]
        extension: Option<String>,
        /// Minimum file size in bytes
        #[arg(long)]
        min_size: Option<u64>,
        /// Maximum file size in bytes
        #[arg(long)]
        max_size: Option<u64>,
        /// Maximum search depth
        #[arg(short, long, default_value_t = 10)]
        depth: usize,
    },
    /// Convert between file formats
    Convert {
        /// Input file
        input: PathBuf,
        /// Output file (optional, derives from input if not provided)
        output: Option<PathBuf>,
        /// Source format (auto-detected if not specified)
        #[arg(long)]
        from: Option<Format>,
        /// Target format
        #[arg(long)]
        to: Format,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum OutputFormat {
    Json,
    Yaml,
    Table,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Format {
    Json,
    Toml,
    Yaml,
    Csv,
}

// CLI implementation
fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    if cli.verbose {
        println!("Verbose mode enabled");
    }
    
    if let Some(config_path) = &cli.config {
        println!("Using config file: {}", config_path.display());
        // Load configuration
    }
    
    match &cli.command {
        Commands::Copy { source, destination, recursive, force } => {
            copy_command(source, destination, *recursive, *force)?;
        }
        Commands::Find { path, name, extension, min_size, max_size, depth } => {
            find_command(path, name.as_deref(), extension.as_deref(), 
                        *min_size, *max_size, *depth, &cli.format)?;
        }
        Commands::Convert { input, output, from, to } => {
            convert_command(input, output.as_ref(), from.as_ref(), to)?;
        }
    }
    
    Ok(())
}

// Command implementations
fn copy_command(
    source: &PathBuf,
    dest: &PathBuf,
    recursive: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Copying {} to {} (recursive: {}, force: {})",
             source.display(), dest.display(), recursive, force);
    
    if !source.exists() {
        return Err(format!("Source path does not exist: {}", source.display()).into());
    }
    
    if dest.exists() && !force {
        return Err("Destination exists and --force not specified".into());
    }
    
    if source.is_file() {
        fs::copy(source, dest)?;
        println!("File copied successfully");
    } else if source.is_dir() && recursive {
        copy_dir_recursive(source, dest)?;
        println!("Directory copied successfully");
    } else {
        return Err("Use --recursive flag to copy directories".into());
    }
    
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_file() {
            fs::copy(&src_path, &dst_path)?;
        } else if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}

fn find_command(
    path: &PathBuf,
    name_pattern: Option<&str>,
    extension: Option<&str>,
    min_size: Option<u64>,
    max_size: Option<u64>,
    max_depth: usize,
    output_format: &OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    find_files_recursive(path, name_pattern, extension, min_size, max_size, 
                        max_depth, 0, &mut results)?;
    
    match output_format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&results)?;
            println!("{}", json);
        }
        OutputFormat::Table => {
            println!("{:<50} {:<10} {:<20}", "Path", "Size", "Modified");
            println!("{:-<80}", "");
            for result in results {
                println!("{:<50} {:<10} {:<20}", result.path, result.size, result.modified);
            }
        }
        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&results)?;
            println!("{}", yaml);
        }
    }
    
    Ok(())
}

#[derive(Serialize, Debug)]
struct FindResult {
    path: String,
    size: u64,
    modified: String,
    is_file: bool,
}

fn find_files_recursive(
    dir: &Path,
    name_pattern: Option<&str>,
    extension: Option<&str>,
    min_size: Option<u64>,
    max_size: Option<u64>,
    max_depth: usize,
    current_depth: usize,
    results: &mut Vec<FindResult>,
) -> io::Result<()> {
    if current_depth > max_depth {
        return Ok(());
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        
        let matches = check_file_criteria(&path, &metadata, name_pattern, extension, min_size, max_size);
        
        if matches {
            results.push(FindResult {
                path: path.display().to_string(),
                size: metadata.len(),
                modified: format!("{:?}", metadata.modified().unwrap_or(std::time::UNIX_EPOCH)),
                is_file: path.is_file(),
            });
        }
        
        if path.is_dir() {
            find_files_recursive(&path, name_pattern, extension, min_size, max_size,
                               max_depth, current_depth + 1, results)?;
        }
    }
    
    Ok(())
}

fn check_file_criteria(
    path: &Path,
    metadata: &fs::Metadata,
    name_pattern: Option<&str>,
    extension: Option<&str>,
    min_size: Option<u64>,
    max_size: Option<u64>,
) -> bool {
    // Check name pattern
    if let Some(pattern) = name_pattern {
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            if !filename_str.contains(pattern) {
                return false;
            }
        }
    }
    
    // Check extension
    if let Some(ext) = extension {
        if let Some(file_ext) = path.extension() {
            if file_ext != ext {
                return false;
            }
        } else {
            return false;
        }
    }
    
    // Check file size
    let file_size = metadata.len();
    if let Some(min) = min_size {
        if file_size < min {
            return false;
        }
    }
    
    if let Some(max) = max_size {
        if file_size > max {
            return false;
        }
    }
    
    true
}

fn convert_command(
    input: &PathBuf,
    output: Option<&PathBuf>,
    _from: Option<&Format>,
    to: &Format,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = match output {
        Some(path) => path.clone(),
        None => {
            let mut path = input.clone();
            let new_ext = match to {
                Format::Json => "json",
                Format::Toml => "toml",
                Format::Yaml => "yaml",
                Format::Csv => "csv",
            };
            path.set_extension(new_ext);
            path
        }
    };
    
    println!("Converting {} to {} (format: {:?})",
             input.display(), output_path.display(), to);
    
    // Implementation would depend on the specific formats
    // This is a simplified example
    let content = fs::read_to_string(input)?;
    
    match to {
        Format::Json => {
            // Convert to JSON format
            fs::write(&output_path, format!("{{\"content\": \"{}\"}}", content))?;
        }
        Format::Yaml => {
            // Convert to YAML format
            fs::write(&output_path, format!("content: |\n  {}", content))?;
        }
        _ => {
            return Err("Conversion format not implemented".into());
        }
    }
    
    println!("Conversion complete: {}", output_path.display());
    Ok(())
}
```

## Async File I/O

Using tokio for asynchronous file operations:

```rust
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt, BufReader};

// Async file operations
async fn async_file_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Read entire file asynchronously
    let content = fs::read_to_string("example.txt").await?;
    println!("File content: {}", content);
    
    // Write file asynchronously
    fs::write("output.txt", "Hello from async Rust!").await?;
    
    // Line-by-line reading for large files
    let file = fs::File::open("large_file.txt").await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let mut line_count = 0;
    while let Some(line) = lines.next_line().await? {
        line_count += 1;
        if line_count <= 5 {
            println!("Line {}: {}", line_count, line);
        }
    }
    println!("Total lines: {}", line_count);
    
    Ok(())
}

// Concurrent file processing
async fn concurrent_file_processing(file_paths: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = file_paths.into_iter().map(|path| {
        tokio::spawn(async move {
            match fs::read_to_string(path).await {
                Ok(content) => (path, Ok(content.len())),
                Err(e) => (path, Err(e)),
            }
        })
    });
    
    let results = futures::future::join_all(tasks).await;
    
    for result in results {
        match result? {
            (path, Ok(size)) => println!("File {}: {} bytes", path, size),
            (path, Err(e)) => println!("Error reading {}: {}", path, e),
        }
    }
    
    Ok(())
}

// Async vs Sync performance comparison
async fn performance_comparison() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;
    
    let files = vec!["file1.txt", "file2.txt", "file3.txt"];
    
    // Sync version
    let start = Instant::now();
    for file in &files {
        let _ = std::fs::read_to_string(file);
    }
    let sync_duration = start.elapsed();
    
    // Async version
    let start = Instant::now();
    let tasks = files.iter().map(|file| fs::read_to_string(file));
    let _ = futures::future::join_all(tasks).await;
    let async_duration = start.elapsed();
    
    println!("Sync duration: {:?}", sync_duration);
    println!("Async duration: {:?}", async_duration);
    
    Ok(())
}
```

## Common Pitfalls and Solutions

### 1. Buffer Management

```rust
// BAD: Reading entire file into memory
fn bad_large_file_processing(path: &Path) -> io::Result<()> {
    let content = fs::read_to_string(path)?; // Could exhaust memory
    for line in content.lines() {
        process_line(line);
    }
    Ok(())
}

// GOOD: Streaming file processing
fn good_large_file_processing(path: &Path) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    for line in reader.lines() {
        let line = line?;
        process_line(&line);
    }
    Ok(())
}

fn process_line(_line: &str) {
    // Process individual line
}
```

### 2. Path Handling

```rust
// BAD: String concatenation for paths
fn bad_path_construction() {
    let path = "/home/user".to_string() + "/" + "documents" + "/" + "file.txt";
    // This breaks on Windows and doesn't handle edge cases
}

// GOOD: Using Path and PathBuf
fn good_path_construction() {
    let base = Path::new("/home/user");
    let full_path = base.join("documents").join("file.txt");
    println!("Path: {}", full_path.display());
}
```

## Exercises

### Exercise 1: Log File Analyzer

Create a log file analyzer that processes large log files efficiently:

```rust
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

#[derive(Debug)]
struct LogAnalysis {
    total_lines: usize,
    level_counts: HashMap<String, usize>,
    error_messages: Vec<String>,
    unique_ips: std::collections::HashSet<String>,
}

struct LogAnalyzer;

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer
    }
    
    pub fn analyze_file(&self, path: &Path) -> io::Result<LogAnalysis> {
        // TODO: Implement log file analysis
        // - Read file line by line (don't load entire file into memory)
        // - Parse each log entry
        // - Count entries by log level
        // - Extract error messages
        // - Find unique IP addresses in the logs
        unimplemented!()
    }
    
    fn parse_log_entry(&self, line: &str) -> Option<LogEntry> {
        // TODO: Parse log entry from line
        // Format: "2023-12-01 10:30:45 [INFO] User 192.168.1.100 logged in"
        unimplemented!()
    }
    
    pub async fn analyze_file_async(&self, path: &Path) -> Result<LogAnalysis, Box<dyn std::error::Error>> {
        // TODO: Implement async version
        unimplemented!()
    }
}

// Test your implementation
fn test_log_analyzer() -> io::Result<()> {
    let analyzer = LogAnalyzer::new();
    let analysis = analyzer.analyze_file(Path::new("server.log"))?;
    
    println!("Log Analysis Results:");
    println!("Total lines: {}", analysis.total_lines);
    println!("Level counts: {:#?}", analysis.level_counts);
    println!("Error messages: {}", analysis.error_messages.len());
    println!("Unique IPs: {}", analysis.unique_ips.len());
    
    Ok(())
}
```

### Exercise 2: Configuration Manager

Build a configuration manager that handles multiple formats:

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppConfig {
    server: ServerConfig,
    database: DatabaseConfig,
    logging: LoggingConfig,
    features: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
    timeout_seconds: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LoggingConfig {
    level: String,
    file_path: Option<String>,
    max_file_size: u64,
}

struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(config_path: PathBuf) -> Self {
        ConfigManager { config_path }
    }
    
    pub fn load(&self) -> Result<AppConfig, Box<dyn std::error::Error>> {
        // TODO: Load configuration from file
        // - Detect format from file extension
        // - Support JSON, TOML, and YAML
        // - Handle missing files with default config
        unimplemented!()
    }
    
    pub fn save(&self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Save configuration to file
        // - Use same format as original file
        // - Create backup before overwriting
        unimplemented!()
    }
    
    pub fn merge_from_env(&self, config: &mut AppConfig) {
        // TODO: Override config values from environment variables
        // - Use naming convention like: APP_SERVER_PORT=8080
        unimplemented!()
    }
    
    pub fn validate(&self, config: &AppConfig) -> Result<(), String> {
        // TODO: Validate configuration values
        // - Check port ranges
        // - Validate database URL format
        // - Ensure required fields are present
        unimplemented!()
    }
}

// Test your implementation
fn test_config_manager() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ConfigManager::new(PathBuf::from("config.toml"));
    
    let mut config = manager.load()?;
    manager.merge_from_env(&mut config);
    manager.validate(&config)?;
    
    println!("Loaded config: {:#?}", config);
    
    // Modify and save
    config.server.port = 9090;
    manager.save(&config)?;
    
    Ok(())
}
```

### Exercise 3: File Synchronizer

Create a file synchronization tool:

```rust
#[derive(Debug, Clone)]
struct SyncStats {
    files_copied: usize,
    files_updated: usize,
    files_deleted: usize,
    bytes_transferred: u64,
}

struct FileSynchronizer {
    source: PathBuf,
    destination: PathBuf,
}

impl FileSynchronizer {
    pub fn new(source: PathBuf, destination: PathBuf) -> Self {
        FileSynchronizer { source, destination }
    }
    
    pub fn sync(&self, dry_run: bool) -> Result<SyncStats, Box<dyn std::error::Error>> {
        // TODO: Implement file synchronization
        // - Compare source and destination directories
        // - Identify new, modified, and deleted files
        // - Copy/update/delete files as needed
        // - If dry_run is true, only report what would be done
        unimplemented!()
    }
    
    pub async fn sync_async(&self, dry_run: bool) -> Result<SyncStats, Box<dyn std::error::Error>> {
        // TODO: Implement async version with progress reporting
        unimplemented!()
    }
    
    fn compare_files(&self, source: &Path, dest: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        // TODO: Compare files to determine if sync is needed
        // - Check modification time
        // - Compare file sizes
        // - Optionally compute checksums for content comparison
        unimplemented!()
    }
    
    fn copy_with_progress(&self, source: &Path, dest: &Path) -> Result<u64, Box<dyn std::error::Error>> {
        // TODO: Copy file with progress reporting
        unimplemented!()
    }
}

// Test your implementation
async fn test_file_synchronizer() -> Result<(), Box<dyn std::error::Error>> {
    let syncer = FileSynchronizer::new(
        PathBuf::from("source_folder"),
        PathBuf::from("backup_folder"),
    );
    
    // Dry run first
    let stats = syncer.sync(true)?;
    println!("Dry run results: {:#?}", stats);
    
    // Actually sync
    let stats = syncer.sync_async(false).await?;
    println!("Sync completed: {:#?}", stats);
    
    Ok(())
}
```

## Key Takeaways

1. **Use Buffered I/O**: Always use `BufReader`/`BufWriter` for better performance with small reads/writes
2. **Handle Errors Properly**: File operations can fail; use proper error handling with `Result<T, E>`
3. **Cross-Platform Paths**: Use `Path` and `PathBuf` instead of string concatenation for file paths
4. **Memory Management**: Process large files line-by-line instead of loading everything into memory
5. **Choose the Right Format**: JSON for web APIs, TOML for configuration, binary for performance
6. **Async for I/O Bound**: Use async file operations when dealing with many files or network storage
7. **CLI Best Practices**: Use clap for robust argument parsing and proper help messages
8. **Validate Input**: Always validate file paths and configuration values before processing

**Next**: In Chapter 16, we'll explore no_std programming for embedded systems and performance-critical applications.