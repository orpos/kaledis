function love.load()
    -- This will run once when the game loads
    -- you should check the .build folder to see that the types are imported
    -- when you remove any of these it changes accordingly
    print(Socket.dns.gethostname())
    print(Enet.host_create("localhost:1234", 1, 2, 2, 2))
end

function love.update(dt)
    -- This will run every frame
end

function love.draw()
    -- This will run every time a new frame will be drawn
end
