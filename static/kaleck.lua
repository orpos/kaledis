-- List of built-in Lua libraries that should never be cleared from package cache
local builtin_libs = {
	string = true,
	table = true,
	math = true,
	io = true,
	os = true,
	debug = true,
	coroutine = true,
	package = true,
	utf8 = true,
	bit = true,
	jit = true,
	ffi = true,
	-- LOVE built-in modules
	love = true,
	kalec = true,
	__polyfill__ = true,
}

function clear_packages()
	for k, _ in pairs(package.loaded) do
		-- Only clear non-builtin packages and those not in the ignorePackages list
		if k:sub(0, 5) ~= "love." then
			if not builtin_libs[k] then
				package.loaded[k] = nil
			end
		end
	end
end

local should_render_error = false
local thread = love.thread.newThread([[
function split_once(str, sep)
    local i, j = string.find(str, sep, 1, true)
    if not i then
        return str, nil
    end

    return string.sub(str, 1, i - 1),
        string.sub(str, j + 1)
end

local socket = require("socket")

local local_channel, channel = ...

local tcp = socket.tcp()
tcp:bind("127.0.0.1", 9532)
tcp:listen(32)
tcp:settimeout(0)
local connection = nil
local buffer = ""
while true do
    if connection == nil then
        connection = tcp:accept()
        socket.sleep(0.001)
        if connection ~= nil then
            connection:settimeout(0.001)
        end
    else
		local msg = local_channel:pop()
		if msg ~= nil then
			connection:send(msg)
		end
        local data, err, partial = connection:receive(4028)
        if err == "closed" then
            connection = nil;
            channel:push("has_server\n"..love.data.compress("string", "gzip", "0", 4))
        else
            if data then
                buffer = buffer..data
            end
            if partial then
                buffer = buffer..partial
            end
            
            local previous_message, rest = split_once(buffer, "-_-EOF-_-")
            if rest ~= nil then
                buffer = rest
                channel:push(previous_message)
            end
        end
    end

    socket.sleep(0.001)
end
]])

function lerp(a, b, t)
	return a + (b - a) * t
end

function mapRange(val, minInput, maxInput, minOutput, maxOutput)
	return minOutput + (val - minInput) * (maxOutput - minOutput) / (maxInput - minInput)
end

local t = 0
function loading_draw(dt, size)
	-- love.graphics.clear()
	t = t + dt
	t = t % 1

	local a, b, c, d = love.graphics.getColor()

	local x, y, w, h = love.window.getSafeArea()
	local center_x = (x + w / 2)
	local center_y = (y + h / 2)

	local thickness = size / 5
	local lat = (size - thickness) / 2
	local offset = lat - thickness
	local radius = size / 10
	love.graphics.push()
	love.graphics.translate(center_x, center_y)
	love.graphics.rotate(math.rad(165))

	love.graphics.setColor(0.86, 0.62, 0.86, 1)

	local w_h, x1_h, x2_h

	if t < 0.35 then
		local localT = mapRange(t, 0, 0.35, 0, 1)
		w_h = lerp(thickness, size, localT)
		x1_h = lerp(lat, 0, localT)
		x2_h = lerp(-lat, 0, localT)
	elseif t < 0.70 then
		local localT = mapRange(t, 0.35, 0.70, 0, 1)
		w_h = lerp(size, thickness, localT)
		x1_h = lerp(0, -lat, localT)
		x2_h = lerp(0, lat, localT)
	else
		local localT = mapRange(t, 0.70, 1.0, 0, 1)
		w_h = thickness
		x1_h = lerp(-lat, lat, localT)
		x2_h = lerp(lat, -lat, localT)
	end

	love.graphics.rectangle("fill", x1_h - w_h / 2, -offset - thickness / 2, w_h, thickness, radius, radius)
	love.graphics.rectangle("fill", x2_h - w_h / 2, offset - thickness / 2, w_h, thickness, radius, radius)

	love.graphics.setColor(0.2, 0.6, 1, 1)

	local h_v, y1_v, y2_v

	if t < 0.35 then
		local localT = mapRange(t, 0, 0.35, 0, 1)
		h_v = lerp(thickness, size, localT)
		y1_v = lerp(lat, 0, localT)
		y2_v = lerp(-lat, 0, localT)
	elseif t < 0.70 then
		local localT = mapRange(t, 0.35, 0.70, 0, 1)
		h_v = lerp(size, thickness, localT)
		y1_v = lerp(0, -lat, localT)
		y2_v = lerp(0, lat, localT)
	else
		local localT = mapRange(t, 0.70, 1.0, 0, 1)
		h_v = thickness
		y1_v = lerp(-lat, lat, localT)
		y2_v = lerp(lat, -lat, localT)
	end

	love.graphics.rectangle("fill", offset - thickness / 2, y1_v - h_v / 2, thickness, h_v, radius, radius)
	love.graphics.rectangle("fill", -offset - thickness / 2, y2_v - h_v / 2, thickness, h_v, radius, radius)

	love.graphics.pop()
	love.graphics.setColor(a, b, c, d)
	-- love.graphics.present()
end

local __llk = {
	local_channel = nil,
	channel = nil,
}
function report_error(error)
	print(error)
end

local h_cn = false
local is_loading_new_version = false

local pool = {
	messages = {},
	size = 100,
	callbacks = {},
}

function pool:new(size)
	local process = setmetatable({}, { __index = self })
	process.messages = {}
	process.size = size
	process.callbacks = {}
	return process
end

function pool:on(type, callback)
	if not pool.callbacks[type] then
		self.callbacks[type] = {}
	end
	local pool = self.callbacks[type]
	table.insert(pool, callback)
end

function pool:add_message(message)
	local type_, data = message:match("([^\n]*)\n(.+)")
	local callbacks = self.callbacks[type_]
	if callbacks ~= nil then
		local function decompress()
			return love.data.decompress("string", "gzip", data)
		end
		local ok, decompressed = pcall(decompress)
		if ok then
			for count = 1, #callbacks, 1 do
				callbacks[count](decompressed)
				-- if not ok then
				-- 	print("Failed to load file")
				-- 	print(a)
				-- end
			end
		end
	end
end

function mysplit(inputstr, sep)
	if sep == nil then
		sep = "%s"
	end
	local t = {}
	for str in string.gmatch(inputstr, "([^" .. sep .. "]+)") do
		table.insert(t, str)
	end
	return t
end

local function handle(err)
	return "ERROR: " .. err
end

local function reloadFile(file)
	local success, chunk = pcall(love.filesystem.load, file)
	if not success then
		print("Failed to load new chunk")
		return
	end
	if chunk then
		local ok, err = xpcall(chunk, handle)
		if not ok then
			print(err)
			return "no"
		end
	end
	return "yes"
end

pool:on("update", function(message)
	local msgs = mysplit(message, ",")
	is_loading_new_version = true
	-- re = false
	should_render_error = false
	for msg_Id = 0, #msgs, 1 do
		local msg = msgs[msg_Id]
		is_loading_new_version = true
		clear_packages()
		print("Reloading")
		if reloadFile("main.lua") == "no" then
			print("Failed to load main")
		end
		-- if reloadFile(msg) then
		-- 	print("Failed to load ", msg)
		-- end
		is_loading_new_version = false
	end
end)

pool:on("has_server", function(has)
	h_cn = has == "1"
	if not h_cn then
		is_loading_new_version = false
	end
end)

function love.run()
	__llk.local_channel = love.thread.newChannel()
	__llk.channel = love.thread.newChannel()

	thread:start(__llk.local_channel, __llk.channel)

	if love.load then
		love.load(love.arg.parseGameArguments(arg), arg)
	end

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
	if love.timer then
		love.timer.step()
	end

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

		while true do
			local message = __llk.channel:pop()
			if not message then
				break
			end
			pool:add_message(message)
		end

		-- Call update and draw
		if love.update and should_render_error == false then
			local ok, error = pcall(function()
				love.update(dt)
			end)
			if not ok then
				should_render_error = true
				report_error(error)
			end
		end -- will pass 0 if love.timer is disabled
		if love.graphics and love.graphics.isActive() then
			love.graphics.origin()
			love.graphics.clear(love.graphics.getBackgroundColor())

			if should_render_error then
				love.graphics.print("An error occurred check your console")
			else
				if love.draw and should_render_error == false then
					local ok, error = pcall(love.draw)
					if not ok then
						should_render_error = true
						report_error(error)
					end
				end
			end
			if is_loading_new_version then
				loading_draw(dt, 100)
			end

			love.graphics.present()
		end

		if love.timer then
			love.timer.sleep(0.001)
		end
	end
end
