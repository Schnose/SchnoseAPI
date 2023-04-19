async function main() {
	const filters = [];

	const kzt_tp = await fetch("https://kztimerglobal.com/api/v2/record_filters?mode_ids=200&has_teleports=true&tickrates=128&limit=99999").then((res) => res.json());
	const kzt_pro = await fetch("https://kztimerglobal.com/api/v2/record_filters?mode_ids=200&has_teleports=false&tickrates=128&limit=99999").then((res) => res.json());
	const skz_tp = await fetch("https://kztimerglobal.com/api/v2/record_filters?mode_ids=201&has_teleports=true&tickrates=128&limit=99999").then((res) => res.json());
	const skz_pro = await fetch("https://kztimerglobal.com/api/v2/record_filters?mode_ids=201&has_teleports=false&tickrates=128&limit=99999").then((res) => res.json());
	const vnl_tp = await fetch("https://kztimerglobal.com/api/v2/record_filters?mode_ids=202&has_teleports=true&tickrates=128&limit=99999").then((res) => res.json());
	const vnl_pro = await fetch("https://kztimerglobal.com/api/v2/record_filters?mode_ids=202&has_teleports=false&tickrates=128&limit=99999").then((res) => res.json());

	for (const filter of kzt_tp) {
		const course_id = (filter.map_id * 1000) + filter.stage;
		if (course_id < 0) {
			continue;
		}
		filters.push({
			course_id,
			mode_id: 200,
		})
	}

	for (const filter of kzt_pro) {
		const course_id = (filter.map_id * 1000) + filter.stage;
		if (course_id < 0) {
			continue;
		}
		filters.push({
			course_id,
			mode_id: 200,
		})
	}

	for (const filter of skz_tp) {
		const course_id = (filter.map_id * 1000) + filter.stage;
		if (course_id < 0) {
			continue;
		}
		filters.push({
			course_id,
			mode_id: 201,
		})
	}

	for (const filter of skz_pro) {
		const course_id = (filter.map_id * 1000) + filter.stage;
		if (course_id < 0) {
			continue;
		}
		filters.push({
			course_id,
			mode_id: 201,
		})
	}

	for (const filter of vnl_tp) {
		const course_id = (filter.map_id * 1000) + filter.stage;
		if (course_id < 0) {
			continue;
		}
		filters.push({
			course_id,
			mode_id: 202,
		})
	}

	for (const filter of vnl_pro) {
		const course_id = (filter.map_id * 1000) + filter.stage;
		if (course_id < 0) {
			continue;
		}
		filters.push({
			course_id,
			mode_id: 202,
		})
	}

	const first_filter = filters.splice(0, 1)[0];

	let sql_string = `INSERT INTO filters
  (course_id, mode_id)
VALUES
  (${first_filter.course_id}, ${first_filter.mode_id})`;

	for (const filter of filters) {
		sql_string += `\n ,(${filter.course_id}, ${filter.mode_id})`;
	}

	sql_string += ";";

	console.log(sql_string);
}

main()
