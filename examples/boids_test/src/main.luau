-- Credit: https://gist.github.com/BradFitz66/e5a9dcc69570ccb9717f6be7e86d8551#file-boids-luau
type Boid = {
	position: vector,
	velocity: vector,
	quad: any,
}

type Camera = {
	position: vector,
	zoom: number,
}
type SpatialCell<T> = {
	items: { T },
}

type SpatialHash<T> = {
	cellSize: number,
	cells: { [vector]: SpatialCell<T> },

	insert: (self: SpatialHash<T>, item: T, position: vector) -> (),
	remove: (self: SpatialHash<T>, item: T, position: vector) -> (),
	query: (self: SpatialHash<T>, position: vector, range: number) -> { T },
	update: (self: SpatialHash<T>, item: T, oldPosition: vector, newPosition: vector) -> (),
}

type BoidArray = {
	boids: { Boid },
	grid: SpatialHash<Boid>,
	update: (self: BoidArray, dt: number) -> (),
	draw: (self: BoidArray) -> (),
}

SCREEN = vector.create(800, 600)

SEPARATION_DISTANCE = 20
ALIGNMENT_STRENGTH = 1
COHESION_STRENGTH = 1.0
SEPARATION_STRENGTH = 10.0
MAX_SPEED = 100
MIN_SPEED = 50
MAX_ACCELERATION = 50
NUMBER_OF_BOIDS = 1000

local grid: SpatialHash<Boid>
local spriteBatch
function vector_limit(v: vector, maxLength: number): vector
	local len: number = vector.magnitude(v) :: number
	if len > maxLength then
		return vector.normalize(v) * maxLength
	end
	return v
end

local camera: Camera = {
	position = vector.create(0, 0),
	zoom = 1.0,
}

local boids: BoidArray = {
	boids = {},
	grid = nil,
	update = function(self, dt)
		local avgVelocity: vector = vector.create(0, 0)
		local avgPosition: vector = vector.create(0, 0)
		local separation: vector = vector.create(0, 0)
		local steer: vector = vector.create(0, 0)
		local count: number = 0
		-- Update boid positions and velocities here
		for i, boid in ipairs(self.boids) do
			steer = vector.zero
			avgVelocity = vector.zero
			avgPosition = vector.zero
			separation = vector.zero
			count = 0
			local oldPosition = boid.position
			boid.position = boid.position + boid.velocity * dt
			local neighbors: { Boid } = self.grid:query(boid.position, 25)
			if #neighbors > 0 then
				for j, neighbor in ipairs(neighbors) do
					--Alignment
					if neighbor ~= boid then
						count += 1
						avgVelocity = avgVelocity + neighbor.velocity
						avgPosition = avgPosition + neighbor.position
						local distance: number = vector.magnitude(boid.position - neighbor.position)
						if distance > 0 and distance < SEPARATION_DISTANCE then
							local diff: vector = vector.normalize(boid.position - neighbor.position)
							diff /= distance
							separation = separation + diff
						end
					end
				end
				if count > 0 then
					--Average the forces.
					avgVelocity = avgVelocity / count
					avgPosition = avgPosition / count
					separation = separation / count

					avgVelocity = vector.normalize(avgVelocity) * MAX_SPEED
					avgPosition = vector.normalize((avgPosition - boid.position))

					local desired_velocity: vector = vector.normalize(avgPosition) * MAX_SPEED

					--Steering force = desired velocity - current velocity
					local steer_alignment: vector = vector_limit((avgVelocity - boid.velocity), MAX_ACCELERATION)

					local steer_cohesion: vector = vector_limit((desired_velocity - boid.velocity), MAX_ACCELERATION)

					steer += steer_alignment * ALIGNMENT_STRENGTH
					steer += steer_cohesion * COHESION_STRENGTH
				end
				if vector.magnitude(separation) > 0 then
					separation = vector.normalize(separation) * MAX_SPEED
					separation = vector_limit((separation - boid.velocity), MAX_ACCELERATION)
					steer += separation * SEPARATION_STRENGTH
				end
			end

			--Add steer to velocity
			boid.velocity = boid.velocity + vector_limit(steer, MAX_ACCELERATION) * dt
			self.grid:update(boid, oldPosition, boid.position)
		end
	end,
	draw = function(self)
		spriteBatch:clear()
		for i, boid in ipairs(self.boids) do
			spriteBatch:add(boid.quad, boid.position.x, boid.position.y)
		end
		love.graphics.draw(spriteBatch, -camera.position.x, -camera.position.y, 0, camera.zoom, camera.zoom)
	end,
}
function love.load()
	spriteBatch = love.graphics.newSpriteBatch(love.graphics.newImage("assets/Boid.png"), NUMBER_OF_BOIDS)
	-- Initialize boids
	love.graphics.setPointSize(3)

	boids.grid = {
		cellSize = 100,
		cells = {},
		insert = function(self, item, position)
			local cellX = math.floor(position.x / self.cellSize)
			local cellY = math.floor(position.y / self.cellSize)
			local key = tostring(cellX) .. "," .. tostring(cellY)
			if not self.cells[key] then
				self.cells[key] = { items = {} }
			end
			table.insert(self.cells[key].items, item)
		end,

		query = function(self, position, range)
			local results: { Boid } = {}
			local minCellX = math.floor((position.x - range) / self.cellSize)
			local maxCellX = math.floor((position.x + range) / self.cellSize)
			local minCellY = math.floor((position.y - range) / self.cellSize)
			local maxCellY = math.floor((position.y + range) / self.cellSize)
			for cellX = minCellX, maxCellX do
				for cellY = minCellY, maxCellY do
					local key = tostring(cellX) .. "," .. tostring(cellY)
					if self.cells[key] then
						for _, item in ipairs(self.cells[key].items) do
							table.insert(results, item)
						end
					end
				end
			end
			return results
		end,
		remove = function(self, item, position)
			local cellX = math.floor(position.x / self.cellSize)
			local cellY = math.floor(position.y / self.cellSize)
			local key = tostring(cellX) .. "," .. tostring(cellY)
			--Delete cell if it has no items left
			if self.cells[key] then
				for i, cellItem in ipairs(self.cells[key].items) do
					if cellItem == item then
						table.remove(self.cells[key].items, i)
						break
					end
				end
				if #self.cells[key].items == 0 then
					self.cells[key] = nil
				end
			end
		end,
		update = function(self, item, oldPosition, newPosition)
			self:remove(item, oldPosition)
			self:insert(item, newPosition)
		end,
	}

	boids.boids = table.create(NUMBER_OF_BOIDS)
	for i = 1, NUMBER_OF_BOIDS do
		boids.boids[i] = {
			position = vector.create(math.random(0, SCREEN.x), math.random(0, SCREEN.y)),
			velocity = vector.create(math.random(-100, 100), math.random(-100, 100)),
			quad = love.graphics.newQuad(0, 0, 8, 8, 8, 8),
		}
		boids.boids[i].velocity = vector.normalize(boids.boids[i].velocity) * 50
		boids.grid:insert(boids.boids[i], boids.boids[i].position)
	end
end

function love.draw()
	-- Draw boids
	boids:draw()
end

function love.update(dt)
	love.window.setTitle("Boids - FPS: " .. tostring(love.timer.getFPS()))

	boids:update(dt)
	if love.keyboard.isDown("up") then
		camera.position = camera.position + vector.create(0, -200) * dt
	end
	if love.keyboard.isDown("down") then
		camera.position = camera.position + vector.create(0, 200) * dt
	end
	if love.keyboard.isDown("left") then
		camera.position = camera.position + vector.create(-200, 0) * dt
	end
	if love.keyboard.isDown("right") then
		camera.position = camera.position + vector.create(200, 0) * dt
	end
end
