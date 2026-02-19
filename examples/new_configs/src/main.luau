local data = require("@packages/byteparse")
local config = require("./config")
local ax, ay, aw, ah = love.window.getSafeArea()

i = 0

print(package.loaded["config"].speed)

function love.load()
	x = 0
	-- This will run once when the game loads
	print("hello!")
	-- print(data)
end

function love.update(dt)
	if i == 0 then
		x += 500 * dt
	else
		x -= 500 * dt
	end

	if x <= 0 and i == 1 then
		i = 0
	end
	if x > aw and i == 0 then
		i = 1
	end
	-- This will run every frame
end

function love.draw()
	-- This will run every time a new frame will be drawn
	-- love.graphics.print("in real time it is really fast")
	-- love.graphics.print("like really fast, almost at the speed of tough, but it depends on the file size")
	love.graphics.print(config.speed)

	-- love.graphics.print(a)

	love.graphics.rectangle("fill", 0, 10, x * config.speed, x * config.speed / 2)
	if x > 500 then
		love.graphics.rectangle("fill", 300, 10, 300 - x * config.speed, 300 - x * config.speed / 2)
	end
end
