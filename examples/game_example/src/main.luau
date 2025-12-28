function love.load()
	-- This will run once when the game loads
end


local y = 0
local x = 0
function love.update(dt)
    -- Here we handle the logic for moving the character
	if love.keyboard.isDown("down") then
        y += 500 * dt
    end
    if love.keyboard.isDown("up") then
        y -= 500 * dt
    end
    if love.keyboard.isDown("right") then
        x += 500 * dt
    end
    if love.keyboard.isDown("left") then
        x -= 500 * dt
    end
end

function love.draw()
	-- Here we draw the character
    love.graphics.rectangle("fill", x, y, 50, 50)
end
