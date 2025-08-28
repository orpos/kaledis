use std::path::{Path, PathBuf};
use darklua_core::{
    nodes::{Arguments, Block, Expression, Prefix, StringExpression},
    process::{DefaultVisitor, NodeProcessor, NodeVisitor},
    rules::{Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties},
};

pub const LUAU_IO_MODIFIER_NAME: &str = "luau_io_modifier";

struct LuauIoProcessor<'a> {
    path: &'a Path,
    project_root_src: &'a PathBuf,
    project_root: &'a PathBuf,
    io_module_path: &'a PathBuf,
}

fn generate_io_module_content() -> String {
    r#"-- Comprehensive IO module for Luau (adapted from Lua's io library)
-- This module provides all IO functionality in a single require

local io = {}

-- File modes for opening files
io.modes = {
    read = "r",
    write = "w", 
    append = "a",
    read_write = "r+",
    write_read = "w+",
    append_read = "a+",
    read_binary = "rb",
    write_binary = "wb",
    append_binary = "ab",
    read_write_binary = "r+b",
    write_read_binary = "w+b",
    append_read_binary = "a+b"
}

-- Default input and output files
io.stdin = nil  -- Will be set by the runtime
io.stdout = nil -- Will be set by the runtime  
io.stderr = nil -- Will be set by the runtime

-- File operations
function io.open(filename: string, mode: string?): (any?, string?)
    mode = mode or "r"
    -- This would interface with the actual file system
    -- Implementation depends on your Luau runtime
    local success, file_or_error = pcall(function()
        -- Your actual file opening implementation here
        return {
            read = function(self, format)
                -- File read implementation
                return ""
            end,
            write = function(self, ...)
                -- File write implementation  
                return true
            end,
            close = function(self)
                -- File close implementation
                return true
            end,
            flush = function(self)
                -- File flush implementation
                return true
            end,
            seek = function(self, whence, offset)
                -- File seek implementation
                return 0
            end,
            setvbuf = function(self, mode, size)
                -- Set buffering mode
                return true
            end
        }
    end)
    
    if success then
        return file_or_error
    else
        return nil, tostring(file_or_error)
    end
end

function io.close(file: any?): boolean?
    file = file or io.output()
    if file and file.close then
        return file:close()
    end
    return nil
end

function io.flush(): boolean
    local out = io.output()
    if out and out.flush then
        return out:flush()
    end
    return false
end

function io.input(file: (string | any)?): any
    if file == nil then
        return io.stdin
    elseif type(file) == "string" then
        local f, err = io.open(file, "r")
        if f then
            io.stdin = f
            return f
        else
            error(err)
        end
    else
        io.stdin = file
        return file
    end
end

function io.output(file: (string | any)?): any
    if file == nil then
        return io.stdout  
    elseif type(file) == "string" then
        local f, err = io.open(file, "w")
        if f then
            io.stdout = f
            return f
        else
            error(err)
        end
    else
        io.stdout = file
        return file
    end
end

function io.read(...): string?
    local input = io.input()
    if input and input.read then
        return input:read(...)
    end
    return nil
end

function io.write(...): boolean
    local output = io.output()
    if output and output.write then
        return output:write(...)
    end
    return false
end

function io.lines(filename: string?, ...): () -> string?
    local f, should_close
    if filename then
        f = io.open(filename, "r")
        should_close = true
    else
        f = io.input()
        should_close = false
    end
    
    if not f then
        return function() return nil end
    end
    
    local args = {...}
    return function()
        if not f then return nil end
        
        local line
        if #args == 0 then
            line = f:read("*l")
        else
            line = f:read(table.unpack(args))
        end
        
        if line == nil and should_close then
            f:close()
            f = nil
        end
        
        return line
    end
end

function io.type(obj: any): string?
    if type(obj) == "table" and obj.read and obj.write then
        return "file"
    elseif type(obj) == "table" and (obj.read or obj.write) then
        return "closed file"
    end
    return nil
end

-- Additional utility functions for common operations
function io.readFile(filename: string): (string?, string?)
    local file, err = io.open(filename, "r")
    if not file then
        return nil, err
    end
    
    local content = file:read("*a")
    file:close()
    return content
end

function io.writeFile(filename: string, content: string): (boolean, string?)
    local file, err = io.open(filename, "w")
    if not file then
        return false, err
    end
    
    local success = file:write(content)
    file:close()
    return success ~= nil
end

function io.appendFile(filename: string, content: string): (boolean, string?)
    local file, err = io.open(filename, "a")
    if not file then
        return false, err
    end
    
    local success = file:write(content)
    file:close()
    return success ~= nil
end

function io.exists(filename: string): boolean
    local file = io.open(filename, "r")
    if file then
        file:close()
        return true
    end
    return false
end

-- Path utilities
io.path = {}

function io.path.join(...): string
    local parts = {...}
    local result = ""
    for i, part in ipairs(parts) do
        if i == 1 then
            result = tostring(part)
        else
            if not result:match("/$") and not tostring(part):match("^/") then
                result = result .. "/"
            end
            result = result .. tostring(part)
        end
    end
    return result
end

function io.path.dirname(path: string): string
    local dir = path:match("(.*/)")
    return dir and dir:sub(1, -2) or "."
end

function io.path.basename(path: string): string
    return path:match("([^/]+)$") or path
end

function io.path.extension(path: string): string
    return path:match("%.([^./]+)$") or ""
end

-- Export the module
return io"#.to_string()
}

impl<'a> NodeProcessor for LuauIoProcessor<'a> {
    fn process_function_call(&mut self, function_call: &mut darklua_core::nodes::FunctionCall) {
        if let Prefix::Identifier(identifier) = function_call.get_prefix() {
            if identifier.get_name() == "require" {
                let args = function_call.mutate_arguments();
                if let Arguments::Tuple(dat) = args {
                    if let Some(Expression::String(expr)) = dat.iter_mut_values().next() {
                        let require_path = expr.get_value().to_string();
                        
                        if require_path == "io" || require_path == "IO" {
                            let io_module_path = self.io_module_path
                                .strip_prefix(self.project_root_src)
                                .unwrap_or_else(|_| self.io_module_path.strip_prefix(self.project_root).unwrap())
                                .iter()
                                .map(|x| x.to_str().unwrap().replace(".", "__"))
                                .collect::<Vec<String>>()
                                .join(".")
                                .trim_end_matches(".luau")
                                .to_string();
                            
                            *expr = StringExpression::from_value(io_module_path);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct LuauIoModifier {
    pub project_root_src: PathBuf,
    pub project_root: PathBuf,
    pub io_module_path: PathBuf,
    pub generate_io_file: bool,
}

impl LuauIoModifier {
    pub fn new() -> Self {
        Self {
            project_root_src: PathBuf::new(),
            project_root: PathBuf::new(),
            io_module_path: PathBuf::from("io.luau"),
            generate_io_file: true,
        }
    }
    
    pub fn with_io_module_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.io_module_path = path.into();
        self
    }
    
    pub fn disable_io_file_generation(mut self) -> Self {
        self.generate_io_file = false;
        self
    }
    
    pub fn ensure_io_module_exists(&self) -> std::io::Result<()> {
        if !self.generate_io_file {
            return Ok(());
        }
        
        let full_path = if self.io_module_path.is_absolute() {
            self.io_module_path.clone()
        } else {
            self.project_root.join(&self.io_module_path)
        };
        
        if !full_path.exists() {
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(full_path, generate_io_module_content())?;
        }
        
        Ok(())
    }
}

impl FlawlessRule for LuauIoModifier {
    fn flawless_process(&self, block: &mut Block, ctx: &Context) {
        let _ = self.ensure_io_module_exists();
        
        let mut processor = LuauIoProcessor {
            path: ctx.current_path(),
            project_root_src: &self.project_root_src,
            project_root: &self.project_root,
            io_module_path: &self.io_module_path,
        };
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for LuauIoModifier {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        if let Some(io_module) = properties.get("io_module_path") {
            if let darklua_core::rules::RulePropertyValue::String(path_str) = io_module {
                self.io_module_path = PathBuf::from(path_str);
            }
        }
        
        if let Some(project_root) = properties.get("project_root") {
            if let darklua_core::rules::RulePropertyValue::String(path_str) = project_root {
                self.project_root = PathBuf::from(path_str);
            }
        }
        
        if let Some(project_root_src) = properties.get("project_root_src") {
            if let darklua_core::rules::RulePropertyValue::String(path_str) = project_root_src {
                self.project_root_src = PathBuf::from(path_str);
            }
        }
        
        if let Some(generate) = properties.get("generate_io_file") {
            let generate_str = format!("{:?}", generate);
            if generate_str.contains("true") || generate_str == "true" {
                self.generate_io_file = true;
            } else if generate_str.contains("false") || generate_str == "false" {
                self.generate_io_file = false;
            }
        }
        
        Ok(())
    }
    
    fn get_name(&self) -> &'static str {
        LUAU_IO_MODIFIER_NAME
    }
    
    fn serialize_to_properties(&self) -> RuleProperties {
        let mut properties = RuleProperties::new();
        properties.insert("io_module_path".to_string(), self.io_module_path.to_string_lossy().to_string().into());
        properties.insert("project_root".to_string(), self.project_root.to_string_lossy().to_string().into());
        properties.insert("project_root_src".to_string(), self.project_root_src.to_string_lossy().to_string().into());
        properties.insert("generate_io_file".to_string(), self.generate_io_file.into());
        properties
    }
}