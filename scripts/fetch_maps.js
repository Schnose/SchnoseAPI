async function main() {
	const api_maps = await fetch("https://kztimerglobal.com/api/v2/maps?limit=9999").then((res) => res.json());
	const kzgo_maps = await fetch("https://kzgo.eu/api/maps").then((res) => res.json());

	/** `maps` SQL Schema
	 * +-------------+-------------------+------+-----+---------+-------+
	 * | Field       | Type              | Null | Key | Default | Extra |
	 * +-------------+-------------------+------+-----+---------+-------+
	 * | id          | smallint unsigned | NO   | PRI | NULL    |       |
	 * | name        | varchar(255)      | NO   |     | NULL    |       |
	 * | global      | tinyint(1)        | NO   |     | 0       |       |
	 * | filesize    | int unsigned      | NO   |     | NULL    |       |
	 * | approved_by | int unsigned      | NO   |     | NULL    |       |
	 * | workshop_id | int unsigned      | YES  |     | NULL    |       |
	 * | created_on  | timestamp         | NO   |     | NULL    |       |
	 * | updated_on  | timestamp         | NO   |     | NULL    |       |
	 * +-------------+-------------------+------+-----+---------+-------+
	 */
	const maps = new Map();

	/** `courses` SQL Schema
	 * +--------+-------------------+------+-----+---------+-------+
	 * | Field  | Type              | Null | Key | Default | Extra |
	 * +--------+-------------------+------+-----+---------+-------+
	 * | id     | int unsigned      | NO   | PRI | NULL    |       |
	 * | map_id | smallint unsigned | NO   |     | NULL    |       |
	 * | stage  | tinyint unsigned  | NO   |     | NULL    |       |
	 * | tier   | tinyint unsigned  | NO   |     | NULL    |       |
	 * +--------+-------------------+------+-----+---------+-------+
	 */
	const courses = [];

	for (const map of api_maps) {
		let created_on = map.created_on;
		if (created_on == "0001-01-01T00:00:00") {
			created_on = "2018-01-09T10:45:49";
		}

		let approved_by = 0;
		if (BigInt(map.approved_by_steamid64) > 76561197960265728n) {
			approved_by = parseInt(BigInt(map.approved_by_steamid64) - 76561197960265728n);
		}

		maps.set(map.name, {
			id: map.id,
			name: map.name,
			global: map.validated,
			filesize: map.filesize,
			approved_by,
			workshop_id: null,
			created_on,
			updated_on: map.updated_on,
		});
	}

	for (const kzgo_map of kzgo_maps) {
		const map = maps.get(kzgo_map.name);
		map.workshop_id = parseInt(kzgo_map.workshopId);

		for (let stage = 0; stage <= kzgo_map.bonuses; stage++) {
			courses.push({
				id: (map.id * 1000) + stage,
				map_id: map.id,
				stage,
				tier: kzgo_map.tier,
			});
		}
	}

	const map_array = Array.from(maps.values());
	map_array.sort((a, b) => a.id - b.id);
	const first_map = map_array.splice(0, 1)[0];

	let sql_string = `INSERT INTO maps
  (id, name, global, filesize, approved_by, workshop_id, created_on, updated_on)
VALUES
  (${first_map.id}, "${first_map.name}", ${first_map.global}, ${first_map.filesize}, ${first_map.approved_by}, ${first_map.workshop_id}, "${first_map.created_on}", "${first_map.updated_on}")`;

	for (const map of map_array) {
		sql_string += `\n ,(${map.id}, "${map.name}", ${map.global}, ${map.filesize}, ${map.approved_by}, ${map.workshop_id}, "${map.created_on}", "${map.updated_on}")`;
	}

	const first_course = courses.splice(0, 1)[0];

	sql_string += `;

INSERT INTO courses
  (id, map_id, stage, tier)
VALUES
  (${first_course.id}, ${first_course.map_id}, ${first_course.stage}, ${first_course.tier})`;

	for (const course of courses) {
		sql_string += `\n ,(${course.id}, ${course.map_id}, ${course.stage}, ${course.tier})`;
	}

	sql_string += ";";

	console.log(sql_string);
}

main()
