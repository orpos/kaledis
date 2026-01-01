local x,y = 0,0
function love.load()
	-- This will run once when the game loads
    y = 0
    x = 0
end

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
    -- love.graphics.print("now it's acceptable")
    love.graphics.rectangle("fill", x, y, 55, 55)
end
