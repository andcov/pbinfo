#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn test_extract_id_from_json() {
        assert_eq!(
            extract_id_from_json("Problema #1691: <strong>Arbore1</strong"),
            Ok(1691)
        );
        assert_eq!(
            extract_id_from_json("Proema #1691: <strong>Arbore1</strong"),
            Err(PbInfoError::JSONError(String::from("The JSON 'label' attribute should be of the form `'Problema #{id}: <strong>{name}</strong>'`")
        )));
        assert_eq!(
            extract_id_from_json("Problema "),
            Err(PbInfoError::JSONError(String::from("The JSON 'label' attribute should be of the form `'Problema #{id}: <strong>{name}</strong>'`")
        )));
    }

    #[test]
    fn text_extract_io() {
        let metadata_file = r#"			</td>
		<td class="center">
			9		</td>
		<td>
			<span style="background: url('/img/32-fisier.png') no-repeat 3px center;background-size:16px;padding-left:34px;"> numere8.in / numere8.out </span> 		</td>
		<td>
					</td>
		<td cass="center""#;
        assert_eq!(
            extract_input_source(&metadata_file),
            Ok(IOSource::File(String::from("numere8.in")))
        );
        assert_eq!(
            extract_output_source(&metadata_file),
            Ok(IOSource::File(String::from("numere8.out")))
        );

        let metadata_std = r#"<td class="center">
			9		</td>
		<td>
			<span style="background: url('/img/32-terminal.png') no-repeat 3px center;background-size:16px;padding-left:34px;">   tastatură / ecran</span>		</td>
		<td>
			0.1 secunde
		</td>
		<td>"#;
        assert_eq!(extract_input_source(&metadata_std), Ok(IOSource::Std));
        assert_eq!(extract_output_source(&metadata_std), Ok(IOSource::Std));
    }

    const IO_TEXT_1: &str = r#"<table class="table table-bordered">
	<tr>
				<th>Postată de</th>
		<th>Clasa</th>
		<th>Intrare/ieșire</th>
		<th>Limită timp</th>
		<th>Limită memorie</th>
		<th>Sursa problemei</th>
		<th>Autor</th>
		<th>Dificultate</th>
				<th>Scorul tău</th>
			</tr>
	<tr>
				<td>
						<span class="pbi-widget-user pbi-widget-user-span">
								<a href="/profil/silviu">
								<img src="https://www.gravatar.com/avatar/529e246d070445d00b4c98ced6152ca7?d=wavatar&s=32" style="border-radius:3px;vertical-align: middle;" />
				Candale Silviu (silviu)								</a>
							</span>
					</td>
		<td class="center">
			11		</td>
		<td>
			<span style="background: url('/img/32-fisier.png') no-repeat 3px center;background-size:16px;padding-left:34px;"> arbore1.in / arbore1.out </span> 		</td>
		<td>
			0.5 secunde
		</td>
		<td>
			<span title="Memorie totală">64 MB</span> / <span  title="Dimensiunea stivei">32 MB</span>
		</td>
		<td>
			ONI 2016, clasele XI-XII		</td>
		<td>
			Denis-Gabriel Mită		</td>
		<td class="center">
			concurs		</td>
							<td>
						<div class="center"><a href="/detalii-evaluare/35494272">100</a></div>
					</td>
						</tr>
</table>"#;

    const IO_TEXT_2: &str = r#"<table class="table table-bordered">
	<tr>
				<th>Postată de</th>
		<th>Clasa</th>
		<th>Intrare/ieșire</th>
		<th>Limită timp</th>
		<th>Limită memorie</th>
		<th>Sursa problemei</th>
		<th>Autor</th>
		<th>Dificultate</th>
				<th>Scorul tău</th>
			</tr>
	<tr>
				<td>
						<span class="pbi-widget-user pbi-widget-user-span">
								<a href="/profil/silviu">
								<img src="https://www.gravatar.com/avatar/529e246d070445d00b4c98ced6152ca7?d=wavatar&s=32" style="border-radius:3px;vertical-align: middle;" />
				Candale Silviu (silviu)								</a>
							</span>
					</td>
		<td class="center">
			9		</td>
		<td>
			<span style="background: url('/img/32-fisier.png') no-repeat 3px center;background-size:16px;padding-left:34px;"> numere8.in / numere8.out </span> 		</td>
		<td>
			0.1 secunde
		</td>
		<td>
			<span title="Memorie totală">64 MB</span> / <span  title="Dimensiunea stivei">8 MB</span>
		</td>
		<td>
			<div class="center">-</div>		</td>
		<td>
			<div class="center">-</div>		</td>
		<td class="center">
			ușoară		</td>
							<td>
						<div class="center"> - </div>
					</td>
						</tr>
</table>"#;

    #[test]
    fn test_extract_grade() {
        assert_eq!(extract_grade(IO_TEXT_1), Ok(11));
        assert_eq!(extract_grade(IO_TEXT_2), Ok(9));
    }

    #[test]
    fn text_extract_time_limit() {
        assert_eq!(
            extract_time_limit(IO_TEXT_1),
            Ok(Some("0.5 secunde".to_owned()))
        );
        assert_eq!(
            extract_time_limit(IO_TEXT_2),
            Ok(Some("0.1 secunde".to_owned()))
        );
    }

    #[test]
    fn text_extract_memory_limit() {
        assert_eq!(
            extract_memory_limit(IO_TEXT_1),
            Ok(Some("64 MB / 32 MB".to_owned()))
        );
        assert_eq!(
            extract_memory_limit(IO_TEXT_2),
            Ok(Some("64 MB / 8 MB".to_owned()))
        );
    }

    #[test]
    fn text_extract_source() {
        assert_eq!(
            extract_source(IO_TEXT_1),
            Ok(Some("ONI 2016, clasele XI-XII".to_owned()))
        );
        assert_eq!(extract_source(IO_TEXT_2), Ok(None));
    }

    #[test]
    fn text_extract_author() {
        assert_eq!(
            extract_author(IO_TEXT_1),
            Ok(Some("Denis-Gabriel Mită".to_owned()))
        );
        assert_eq!(extract_author(IO_TEXT_2), Ok(None));
    }
    #[test]
    fn text_extract_difficulty() {
        assert_eq!(
            extract_difficulty(IO_TEXT_1),
            Ok(Some("concurs".to_owned()))
        );
        assert_eq!(extract_difficulty(IO_TEXT_2), Ok(Some("ușoară".to_owned())));
    }
}
