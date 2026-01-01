function love.load()
    -- This will run once when the game loads
    -- This should be true since our polyfill makes the value be always 0.0089
    print(os.clock() == 0.0089)
end

function love.update(dt)
    -- This will run every frame
end

function love.draw()
    -- This will run every time a new frame will be drawn
    love.graphics.print(os.clock())
end