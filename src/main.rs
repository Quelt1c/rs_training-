use std::collections::HashMap;

fn main
{
  let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel nibh massa. Duis convallis elementum felis sit amet elementum. Donec vitae nibh sed lorem luctus consectetur. Fusce aliquam, eros sit amet elementum suscipit, lorem lorem ultricies nibh, sed malesuada augue orci nec dui. Phasellus gravida dolor in nunc pulvinar suscipit. Pellentesque nec molestie tellus. Pellentesque ut diam quis arcu sodales bibendum nec ut lorem. Suspendisse sagittis lacus metus, ut vestibulum quam lacinia a. Ut ac diam eu metus finibus pharetra eu quis erat. Phasellus cursus luctus tempus.";
  let mut count = HashMap::new();

  /*
	for(index, letter) in text.char_indices() {
		if count.contains_key(&letter) {
		count.get_mut(&letter).unwrap().push(index); unwrap more info
		}	
	v1	else{
		let mut new_vector = Vec::new();
		new_vector.push(index);
		count.insert(letter, new_vector); lookup ag
		}
	v2	else{
		count.insert(letter, vec![index]);
		}
		
	}
	println("{:#?}, count");
  */

//v3
  for (index, letter) in text.char_indices(){
    count.entry(letter).or_insert(Vec::new()).push(index); // entry more info
  }

  for(letter, list) in count{
      println!("{}: {:?}", letter, list);
    }
}

//Зробити щоб розбивалися на слова
//split з власним патерном
//текст у файлі
//окремі методи
//аналайзер

  /*
	for(index, letter) in text.char_indices() {
		if count.contains_key(&letter) {
		count.get_mut(&letter).unwrap().push(index); unwrap more info
		}	
	v1	else{
		let mut new_vector = Vec::new();
		new_vector.push(index);
		count.insert(letter, new_vector); lookup ag
		}
	v2	else{
		count.insert(letter, vec![index]);
		}
		
	}
	println("{:#?}, count");
  */
  /*
	for(index, letter) in text.char_indices() {
		if count.contains_key(&letter) {
		count.get_mut(&letter).unwrap().push(index); unwrap more info
		}	
	v1	else{
		let mut new_vector = Vec::new();
		new_vector.push(index);
		count.insert(letter, new_vector); lookup ag
		}
	v2	else{
		count.insert(letter, vec![index]);
		}
		
	}
	println("{:#?}, count");
  */
