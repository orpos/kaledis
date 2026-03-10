local x = 0
local y = 0

function love.load()
	-- This will run once when the game loads
end

function love.update(dt)
	if love.keyboard.isDown("w") then
		y -= 400 * dt
	end
	if love.keyboard.isDown("a") then
		x -= 400 * dt
	end
	if love.keyboard.isDown("s") then
		y += 400 * dt
	end
	if love.keyboard.isDown("d") then
		x += 400 * dt
	end

	-- This will run every frame
end

function love.draw()
	love.graphics.rectangle("fill", x, y, 50, 50)
	-- This will run every time a new frame will be drawn
end
