-- Credit: https://codeberg.org/usysrc/LICK

-- lick.lua
--
-- simple LIVECODING environment for Löve
-- overwrites love.run, pressing all errors to the terminal/console or overlays it
--

local lick = {}
lick.debug = false                     -- show debug output
lick.reset = false                     -- reset the game and call love.load on file change
lick.clearFlag = false                 -- clear the screen on file change
lick.sleepTime = 0.001                 -- sleep time in seconds
lick.showReloadMessage = true          -- show message when a file is reloaded
lick.chunkLoadMessage = "CHUNK LOADED" -- message to show when a chunk is loaded
lick.updateAllFiles = false            -- include files in watchlist for changes
lick.clearPackages = false             -- clear all packages in package.loaded on file change
lick.defaultFile = "main.lua"          -- default file to load
lick.fileExtensions = { ".lua" }       -- file extensions to watch
lick.entryPoint = "main.lua"           -- entry point for the game, if empty, all files are reloaded
lick.debugTextXOffset = 50             -- X offset for debug text from the center (positive moves right)
lick.debugTextWidth = 400              -- Maximum width for debug text
lick.debugTextAlpha = 0.8              -- Opacity of the debug text (0.0 to 1.0)
lick.debugTextAlignment = "right"      -- Alignment of the debug text ("left", "right", "center", "justify")

-- local variables
-- No longer needed, debug_output tracks persistent errors
local last_modified = {}
local debug_output = nil
local working_files = {}
local should_clear_screen_next_frame = false -- Flag to clear screen on next draw cycle

-- Helper to handle error output and update debug_output
local function handleErrorOutput(err_message)
    -- Ensure the message starts with "ERROR: " for console output if it doesn't already
    local console_message = tostring(err_message)
    if not console_message:find("^ERROR: ") then
        console_message = "ERROR: " .. console_message
    end
    print(console_message)

    -- Update debug_output for on-screen display
    if debug_output then
        debug_output = debug_output .. console_message .. "\n"
    else
        debug_output = console_message .. "\n"
    end
end

-- Error handler wrapping for pcall
local function handle(err)
    return "ERROR: " .. err
end

-- Function to collect all files in the directory and subdirectories with the given extensions into a set
local function collectWorkingFiles(file_set, dir)
    dir = dir or ""
    local files = love.filesystem.getDirectoryItems(dir)
    for _, file in ipairs(files) do
        local filePath = dir .. (dir ~= "" and "/" or "") .. file
        local info = love.filesystem.getInfo(filePath)
        if info and info.type == "file" then
            for _, ext in ipairs(lick.fileExtensions) do
                if file:sub(- #ext) == ext then
                    file_set[filePath] = true -- Add to set for uniqueness
                end
            end
        elseif info and info.type == "directory" then
            collectWorkingFiles(file_set, filePath)
        end
    end
end

-- Initialization
local function load()
    -- Clear previous working files to prevent accumulation if load() is called multiple times
    working_files = {}

    if not lick.updateAllFiles then
        table.insert(working_files, lick.defaultFile)
    else
        local file_set = {}
        collectWorkingFiles(file_set, "") -- Start collection from root directory
        -- Convert set to ordered list
        for file_path, _ in pairs(file_set) do
            table.insert(working_files, file_path)
        end
    end

    -- Initialize the last_modified table for all working files
    for _, file in ipairs(working_files) do
        local info = love.filesystem.getInfo(file)
        -- Ensure info exists before accessing modtime; set to 0 or current time if file not found
        if info then
            last_modified[file] = info.modtime
        else
            -- If a file listed in working_files doesn't exist, treat its modtime as 0
            -- This ensures it will appear as "modified" if it ever appears later.
            last_modified[file] = 0
        end
    end
end

local function reloadFile(file)
    local success, chunk = pcall(love.filesystem.load, file)
    if not success then
        handleErrorOutput(chunk)
        return
    end
    if chunk then
        local ok, err = xpcall(chunk, handle)
        if not ok then
            handleErrorOutput(err)
        else
            if lick.showReloadMessage then print(lick.chunkLoadMessage) end
            debug_output = nil
        end
    end

    if lick.reset and love.load then
        local loadok, err = xpcall(love.load, handle)
        if not loadok then -- Always report load errors
            handleErrorOutput(err)
        end
    end
end

-- if a file is modified, reload relevant files
local function checkFileUpdate()
    local any_file_modified = false
    local files_actually_modified = {} -- Store paths of files whose modtime has changed

    for _, file_path in ipairs(working_files) do
        local info = love.filesystem.getInfo(file_path)
        -- Check if file exists and its modification time has changed
        -- Use `or 0` for `last_modified[file_path]` to handle cases where it might not be initialized,
        -- ensuring `info.modtime` (if exists) is always greater than 0.
        if info and info.type == "file" and info.modtime and info.modtime > (last_modified[file_path] or 0) then
            any_file_modified = true
            table.insert(files_actually_modified, file_path)
            last_modified[file_path] = info.modtime -- Update the last modified time
        elseif not info and last_modified[file_path] ~= nil then
            -- Handle case where a previously tracked file no longer exists (it was deleted)
            -- This means its state has changed.
            any_file_modified = true
            last_modified[file_path] = 0 -- Set to 0 so if it reappears, it's detected as modified
            -- Note: We don't add deleted files to `files_actually_modified` because `reloadFile`
            -- would fail if called on a non-existent file. The effect of deletion is usually
            -- handled by re-running the entry point or by the user.
        end
    end

    if not any_file_modified then
        return -- No files changed, nothing to do
    end

    -- If lick.clearFlag is true, set a flag to clear the screen on the next draw
    if lick.clearFlag then
        should_clear_screen_next_frame = true
    end

    -- If any file was modified, clear packages from the require cache if configured
    if lick.clearPackages then
        for k, _ in pairs(package.loaded) do
            package.loaded[k] = nil
        end
    end

    if lick.entryPoint ~= "" then
        -- If an entry point is defined, reload it. This ensures the entire game logic
        -- (which might implicitly depend on modified files) is re-executed.
        reloadFile(lick.entryPoint)
    else
        -- If no specific entry point, only reload the files that were actually modified.
        for _, file_path in ipairs(files_actually_modified) do
            reloadFile(file_path)
        end
    end

    -- last_modified for files that actually changed was updated in the initial loop.
    -- For files that didn't change, their last_modified values remain correct.
    -- If a file was deleted, its last_modified is set to 0.
    -- No further global update loop for last_modified is needed.
end

local function update(dt)
    checkFileUpdate()
    if not love.update then return end
    local updateok, err = pcall(love.update, dt)
    if not updateok then -- Always report update errors
        handleErrorOutput(err)
    end
end

local function draw()
    local drawok, err = xpcall(love.draw, handle)
    if not drawok then -- Always report draw errors
        handleErrorOutput(err)
    end

    if lick.debug and debug_output then
        love.graphics.setColor(1, 1, 1, lick.debugTextAlpha)
        love.graphics.printf(debug_output, (love.graphics.getWidth() / 2) + lick.debugTextXOffset, 0, lick
            .debugTextWidth, lick.debugTextAlignment)
    end
end


function love.run()
    load()
    if love.load then love.load(love.arg.parseGameArguments(arg), arg) end

    -- Workaround for macOS random number generator issue
    -- On macOS, the random number generator can produce the same sequence of numbers
    -- if not properly seeded. This workaround ensures that the random number generator
    -- is seeded correctly to avoid this issue.
    if jit and jit.os == "OSX" then
        math.randomseed(os.time())
        math.random()
        math.random()
    end

    -- We don't want the first frame's dt to include time taken by love.load.
    if love.timer then love.timer.step() end

    local dt = 0

    return function()
        if love.event then
            love.event.pump()
            for name, a, b, c, d, e, f in love.event.poll() do
                if name == "quit" then
                    if not love.quit or not love.quit() then
                        return a or 0
                    end
                end
                love.handlers[name](a, b, c, d, e, f)
            end
        end

        -- Update dt, as we'll be passing it to update
        if love.timer then
            dt = love.timer.step()
        end

        -- Call update and draw
        if update then update(dt) end -- will pass 0 if love.timer is disabled
        if love.graphics and love.graphics.isActive() then
            love.graphics.origin()
            -- Clear the screen based on lick.clearFlag and file modification
            if lick.clearFlag and should_clear_screen_next_frame then
                love.graphics.clear(love.graphics.getBackgroundColor())
                should_clear_screen_next_frame = false -- Reset the flag after clearing
            elseif not lick.clearFlag then
                -- If lick.clearFlag is false, clear the screen every frame (default behavior)
                love.graphics.clear(love.graphics.getBackgroundColor())
            end
            if draw then draw() end
            love.graphics.present()
        end

        if love.timer then love.timer.sleep(lick.sleepTime) end
    end
end

return lick
