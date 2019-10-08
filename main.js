const n = 25
const tileWidth = 32

const canvas = document.getElementById('canvas')
const ctx = canvas.getContext('2d')

let state = new Array(n * n).fill(null);

function start() {

	canvas.width = n * tileWidth
	canvas.height = n * tileWidth

	canvas.style.width = `${n * tileWidth}px`
	canvas.style.height = `${n * tileWidth}px`

	canvas.style.background = '#222'

	for (let i = 0; i < 100; i++) {
		const j = Math.floor(Math.random() * n * n)
		state[j] = {
			team: 0,
			level: 1,
			strategy: disperseStrategy,
		}
	}

	loop()
}

function loop() {
	update()
	draw()

	requestAnimationFrame(loop)
	//setTimeout(loop, 1000)
}

function draw() {
	state.forEach((agent, i) => {
		ctx.resetTransform()

		const row = Math.floor(i / n)
		const col = i % n

		// Background color
		ctx.fillStyle = (row + col) % 2 ? '#222' : '#333'

		ctx.fillRect(col * tileWidth, row * tileWidth, tileWidth, tileWidth);

		if (agent) {
			ctx.translate(col * tileWidth + tileWidth / 2, row * tileWidth + tileWidth / 2)

			const colors = ['#06f']
			ctx.fillStyle = colors[agent.team]
			ctx.fillRect(-10, -10, 20, 20)

			ctx.fillStyle = 'white'
			ctx.fillText(agent.level, 0, 0)
		}
	})
}

function update() {
	const idx = (r, c) => ((r + n) % n) * n + (c + n) % n;

	tempState = new Array(n * n);
	for (let i = 0; i < n * n; i++) {
		tempState[i] = []
	}

	state.forEach((agent, i) => {
		if (!agent) return

		const row = Math.floor(i / n)
		const col = i % n

		const nbd = [ idx(row, col + 1), idx(row - 1, col), idx(row, col - 1), idx(row + 1, col) ]

		const probs = agent.strategy(agent, nbd)
		const action = pick(probs)

		const {dir, amount} = action
		
		if (agent.level - amount > 0) {
			tempState[i].push({
				...agent,
				level: agent.level - amount
			})
		}

		if (amount > 0) {
			const j = idx(row + dir[0], col + dir[1])
			tempState[j].push({
				...agent,
				level: Math.min(amount, agent.level)
			})
		}
	})

	state = tempState.map((agents) => {
		if (agents.length === 0) {
			return null
		}

		agents[0].level = agents.reduce((a, { level }) => a + level, 0)
		return agents[0]
	})

	const totalLevels = state.reduce((a, agent) => agent ? (a + agent.level) : a, 0)
	console.log(totalLevels)
}

function pick(actionTuples) {
	const total = actionTuples.reduce((a, [_, p]) => a + p, 0)
	const r = Math.random() * total
	let acc = 0
	let i = -1
	while (acc < r) {
		i++
		acc += actionTuples[i][1]
	}
	return actionTuples[i][0]
}

function randomWalkStrategy(agent, ndb) { // outputs action probabilities
	return [[1, 0], [0, -1], [-1, 0], [0, 1]].map(dir => [
		{
			dir,
			amount: agent.level,
		},
		1
	])
}

function disperseStrategy(agent, nbd) {
	const dirs = [[0, 1], [-1, 0], [0, -1], [1, 0]]
	return nbd.map((nbr, i) => [{ dir: dirs[i], amount: agent.level }, nbr ? 0.00001 : 1])
}

window.onload = start
