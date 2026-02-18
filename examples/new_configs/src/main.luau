local data = require("@packages/byteparse")
local config = require("./config")


function love.load()
	a = 0
	-- This will run once when the game loads
	print("hello!")
	-- print(data)
end

function love.update(dt)
	a += 10 * dt
	-- This will run every frame
end


function love.draw()
	-- This will run every time a new frame will be drawn
	love.graphics.print(a)
	love.graphics.rectangle("fill", 0, 10, a * config.speed, a * config.speed)
end

