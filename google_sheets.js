function testSend() {
	const verses = [
		{ tag: "out-of-darkness", psg: "1 Peter 2:9" },
		{ tag: "self-control", psg: "1 Peter 4:7" },
		{ tag: "assurance", psg: "1 John 5:13" },
		{ tag: "sight-of-god", psg: "Proverbs 5:21" },
		{ tag: "self-control", psg: "Titus 2:12" },
		{ tag: "spiritual-warfare", psg: "1 Peter 2:11" },
		{ tag: "will-of-god", psg: "1 Peter 2:15" },
	];
	sendVerseMessage(verses);
}

function sendVerseMessage(verses) {
	const today = new Date();
	const year = today.getFullYear();
	const month = today.getMonth();
	const day = today.getDate()

	const dailyVerses = verses;
	const weeklyVerses = verses;
	const monthlyVerses = verses;

	function formatVerses(verses) {
		const longest = Math.max(...verses.map((v) => v.tag.length));
		const content = verses.map((v) => {
			const space = longest + 3;
			return `#${v.tag}`.padEnd(space) + `"${v.psg}"`
		}).join("\n");
		
		const t = "```";
		return `${t}cpp\n${content}\n${t}`
	}

	const daily = formatVerses(dailyVerses);
	const weekly = formatVerses(weeklyVerses);
	const monthly = formatVerses(monthlyVerses);

	const messageContent = {
		embeds: [{
			title: `${year}-${month}-${day} Verses`,
			color: 3433707,
			fields: [
				{
					"name": "Daily",
					"value": daily,
					"inline": false
				},
				{
					"name": "Weekly",
					"value": weekly,
					"inline": false
				},
				{
					"name": "Monthly",
					"value": monthly,
					"inline": false
				},
			]
		}]
	};

	const options = {
		method: "post",
		contentType: "application/json",
		payload: JSON.stringify(messageContent),
		muteHttpExceptions: true // Prevents script from stopping on HTTP errors
	};

	UrlFetchApp.fetch(DISCORD_WEBHOOK_URL, options);
}
