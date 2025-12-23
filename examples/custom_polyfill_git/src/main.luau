function love.load()
    -- Currently dalbit has a bug with the polyfill implementation of pairs
    -- so i am also putting this here to show a workaround found here: https://github.com/yancouto/psych0
    -- the main changes are in the kaledis.toml, it disables certain emulations made by dalbit
    local test = {}
	test["hi"] = true
	test["bye"] = true
	for a,b in pairs(test) do
		print('before', a,b)
	end
	test["la"] = true
	test["bye"] = nil
	for a,b in pairs(test) do
		print('after', a,b)
	end
end

function love.update(dt)
    -- This will run every frame
end

function love.draw()
    -- This will run every time a new frame will be drawn
end
