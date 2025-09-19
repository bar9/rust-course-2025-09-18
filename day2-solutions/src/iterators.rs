// Chapter 11: Iterators - Data Pipeline Solution

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogEntry {
    pub fn parse(line: &str) -> Option<LogEntry> {
        // Format: "timestamp|level|message"
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() != 3 {
            return None;
        }
        
        let timestamp = parts[0].parse().ok()?;
        let level = match parts[1] {
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARNING" => LogLevel::Warning,
            "ERROR" => LogLevel::Error,
            _ => return None,
        };
        
        Some(LogEntry {
            timestamp,
            level,
            message: parts[2].to_string(),
        })
    }
}

pub struct LogAnalyzer<'a> {
    lines: &'a [String],
}

impl<'a> LogAnalyzer<'a> {
    pub fn new(lines: &'a [String]) -> Self {
        LogAnalyzer { lines }
    }
    
    pub fn parse_entries(&self) -> impl Iterator<Item = LogEntry> + '_ {
        self.lines.iter()
            .filter_map(|line| LogEntry::parse(line))
    }
    
    pub fn errors_only(&self) -> impl Iterator<Item = LogEntry> + '_ {
        self.parse_entries()
            .filter(|entry| entry.level == LogLevel::Error)
    }
    
    pub fn in_time_range(&self, start: u64, end: u64) -> impl Iterator<Item = LogEntry> + '_ {
        self.parse_entries()
            .filter(move |entry| entry.timestamp >= start && entry.timestamp <= end)
    }
    
    pub fn count_by_level(&self) -> HashMap<LogLevel, usize> {
        self.parse_entries()
            .fold(HashMap::new(), |mut counts, entry| {
                *counts.entry(entry.level).or_insert(0) += 1;
                counts
            })
    }
    
    pub fn most_recent(&self, n: usize) -> Vec<LogEntry> {
        let mut entries: Vec<LogEntry> = self.parse_entries().collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.timestamp));
        entries.into_iter().take(n).collect()
    }
    
    // Additional useful methods
    pub fn warnings_and_errors(&self) -> impl Iterator<Item = LogEntry> + '_ {
        self.parse_entries()
            .filter(|entry| matches!(entry.level, LogLevel::Warning | LogLevel::Error))
    }
    
    pub fn search_message(&self, keyword: &'a str) -> impl Iterator<Item = LogEntry> + '_ {
        self.parse_entries()
            .filter(move |entry| entry.message.contains(keyword))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_entry() {
        let valid_line = "1000|INFO|Server started";
        let entry = LogEntry::parse(valid_line);
        assert!(entry.is_some());
        
        let entry = entry.unwrap();
        assert_eq!(entry.timestamp, 1000);
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Server started");
    }
    
    #[test]
    fn test_invalid_entry() {
        let invalid_line = "invalid line";
        assert!(LogEntry::parse(invalid_line).is_none());
    }
    
    #[test]
    fn test_log_analyzer() {
        let log_lines = vec![
            "1000|INFO|Server started".to_string(),
            "1001|DEBUG|Connection received".to_string(),
            "1002|ERROR|Failed to connect".to_string(),
            "invalid line".to_string(),
            "1003|WARNING|High memory".to_string(),
            "1004|ERROR|Timeout".to_string(),
        ];
        
        let analyzer = LogAnalyzer::new(&log_lines);
        
        // Test valid entries count
        assert_eq!(analyzer.parse_entries().count(), 5);
        
        // Test errors only
        let errors: Vec<_> = analyzer.errors_only().collect();
        assert_eq!(errors.len(), 2);
        
        // Test count by level
        let counts = analyzer.count_by_level();
        assert_eq!(counts.get(&LogLevel::Error), Some(&2));
        assert_eq!(counts.get(&LogLevel::Info), Some(&1));
        
        // Test time range
        let range_entries: Vec<_> = analyzer.in_time_range(1001, 1003).collect();
        assert_eq!(range_entries.len(), 3);
        
        // Test most recent
        let recent = analyzer.most_recent(2);
        assert_eq!(recent[0].timestamp, 1004);
        assert_eq!(recent[1].timestamp, 1003);
    }
}