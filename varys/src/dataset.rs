use clap::ValueEnum;

use varys_database::database::interaction::Interaction;

#[derive(ValueEnum, Default, Clone, Debug)]
pub enum DatasetSize {
    /// The full, unchanged dataset.
    #[default]
    Full,
    /// A small dataset with 13 queries, each from a different category.
    Small,
    /// A binary dataset with the two queries *"Call John Doe"* and *"Call Mary Poppins"*.
    Binary,
}

impl DatasetSize {
    /// Filter out all interactions that should not be used for this dataset size.
    ///
    /// # Arguments
    ///
    /// * `interactions`: The interactions to filter.
    pub fn filter(&self, interactions: Vec<Interaction>) -> Vec<Interaction> {
        if let DatasetSize::Full = self {
            return interactions;
        }

        interactions
            .into_iter()
            .filter(|interaction| self.queries().contains(&interaction.query.as_str()))
            .collect()
    }

    /// All queries that are used for this dataset size.
    ///
    /// This returns an empty vector if the dataset size is `DatasetSize::Full`.
    pub fn queries(&self) -> Vec<&str> {
        match self {
            DatasetSize::Full => vec![
                // the 13 queries from the small dataset are excluded sice their higher number of
                // samples may skew the training results:
                // "Hey Siri. What’s 2330 dollars in euros?",
                // "Hey Siri. What is the factorial of 6?",
                // "Hey Siri. What day was 90 days ago?",
                // "Hey Siri. What is the temperature in living room?",
                // "Hey Siri. Any missed calls?",
                // "Hey Siri. Read Calendar",
                // "Hey Siri. Remind me to wash the car",
                // "Hey Siri. How far is New York from Boston",
                // "Hey Siri. How old is Ian McKellen?",
                // "Hey Siri. What’s the temperature outside?",
                // "Hey Siri. Translate car from English to Spanish",
                // "Hey Siri. Roll a die",
                // "Hey Siri. Is there a God?",
                "Hey Siri. What are 130 miles in yards?",
                "Hey Siri. What's 200 pounds in kilograms?",
                "Hey Siri. What's 45 miles per hour in meters per second?",
                "Hey Siri. What are 3 gigabytes in megabytes?",
                "Hey Siri. Convert 4.2 acres to square meters.",
                "Hey Siri. Convert 250 milliliters to cups.",
                "Hey Siri. Convert 180 degrees Celsius to Fahrenheit.",
                "Hey Siri. Convert 3000 calories to kilojoules.",
                "Hey Siri. Convert 75 miles per gallon to kilometers per liter.",
                "Hey Siri. What’s 9 plus 53?",
                "Hey Siri. What is 2 to the power of 17?",
                "Hey Siri. What is the result of 25 to the power of 4?",
                "Hey Siri. What is 244 plus 5%?",
                "Hey Siri. What is $200 minus 21%?",
                "Hey Siri. What is 9 percent of 63?",
                "Hey Siri. What is the area of a circle with a radius of 2 meters?",
                "Hey Siri. What is the remainder when 27 is divided by 5?",
                "Hey Siri. Calculate the hypotenuse of a right triangle with legs 3 and 4.",
                "Hey Siri. Find the greatest common divisor of 48 and 36.",
                "Hey Siri. What date is 90 days before December 17?",
                "Hey Siri. What year is 39 years after 1994?",
                "Hey Siri. How many years until 2049?",
                "Hey Siri. How many days until Easter?",
                "Hey Siri. How many days until Christmas?",
                "Hey Siri. What are two hours five minutes and 39 seconds in seconds?",
                "Hey Siri. What is the time zone in London?",
                "Hey Siri. What time is it in London?",
                "Hey Siri. Current time?",
                "Hey Siri. Turn the lights blue",
                "Hey Siri. Turn off the radio",
                "Hey Siri. I’m home",
                "Hey Siri. Set the brightness of the downstairs lights to 50%",
                "Hey Siri. Lock the front door",
                "Hey Siri. Open the garage",
                "Hey Siri. John is my brother",
                "Hey Siri. That’s not how you say John Doe",
                "Hey Siri. Show John Doe",
                "Hey Siri. When is John’s birthday?",
                "Hey Siri. How old is my brother?",
                "Hey Siri. Whose phone is this?",
                "Hey Siri. Learn to pronounce my name",
                "Hey Siri. Call John",
                "Hey Siri. Call 408 555 1212",
                "Hey Siri. Call my brother on speakerphone",
                "Hey Siri. Call the nearest restaurant",
                "Hey Siri. When did my brother call me?",
                "Hey Siri. Play voicemail from John",
                "Hey Siri. Get my call history",
                "Hey Siri. Redial my last number",
                "Hey Siri. Call back my last missed call",
                "Hey Siri. Any new voicemail?",
                "Hey Siri. Play me my latest voicemail",
                "Hey Siri. Show me new messages from John Doe",
                "Hey Siri. Show me my messages",
                "Hey Siri. Read my messages",
                "Hey Siri. Text John Doe I’m in a meeting",
                "Hey Siri. Message my brother I’ll be late",
                "Hey Siri. Send John see you later",
                "Hey Siri. Tell John I’m on the way",
                "Hey Siri. Ask my brother Where are you?",
                "Hey Siri. Any new email from John Doe?",
                "Hey Siri. Show me the email from John Doe yesterday",
                "Hey Siri. Send an email to John Doe Protocol",
                "Hey Siri. Check email",
                "Hey Siri. Read my last email",
                "Hey Siri. Post to Facebook I’m eating a sandwich",
                "Hey Siri. Post to Twitter Happy New Year!",
                "Hey Siri. Tweet with my location very hot here",
                "Hey Siri. Show me tweets from Twitter",
                "Hey Siri. Show me the latest tweets",
                "Hey Siri. Schedule an event Party in New York Wednesday at 10 PM",
                "Hey Siri. Schedule a meeting at 1 PM tomorrow for 2 hours",
                "Hey Siri. Create a recurring event every Saturday at 2:30 PM called Party",
                "Hey Siri. Set up a meeting with John for today at 3 PM",
                "Hey Siri. Show me my next appointment",
                "Hey Siri. Where is my next meeting?",
                "Hey Siri. Show me the appointments for this afternoon",
                "Hey Siri. What does my calendar look like on Monday?",
                "Hey Siri. When am I meeting with John Doe?",
                "Hey Siri. Cancel my Party in New York event from tomorrow",
                "Hey Siri. Cancel my event with John Doe",
                "Hey Siri. Move my Monday meeting with John to 3 o’clock",
                "Hey Siri. Remind me on Friday at 10 PM to wash the car",
                "Hey Siri. Add Milk to the Grocery list",
                "Hey Siri. Remind me to wash the car when I leave home today",
                "Hey Siri. Remind me to buy milk next time I’m here",
                "Hey Siri. Remind me to wash the car every second week",
                "Hey Siri. Delete the reminder wash the car",
                "Hey Siri. Show me my Grocery list",
                "Hey Siri. Note 12 Dollars for pizza",
                "Hey Siri. Note Interesting Movies",
                "Hey Siri. Add 10 Dollars for food to Outcomes note",
                "Hey Siri. Add Star Wars to Interesting Movies note",
                "Hey Siri. Show me my notes",
                "Hey Siri. Show me my note Interesting Movies",
                "Hey Siri. Show me my notes from last week",
                "Hey Siri. Tell me about the traffic in New York",
                "Hey Siri. What are some attractions around here?",
                "Hey Siri. Where is Big Ben?",
                "Hey Siri. Is the Central Park open now?",
                "Hey Siri. Distance between here and New York?",
                "Hey Siri. How far away is Boston?",
                "Hey Siri. What is the nearest restaurant?",
                "Hey Siri. Find a Starbucks",
                "Hey Siri. Good Mexican restaurants around here",
                "Hey Siri. Table for two in Palo Alto tonight",
                "Hey Siri. Make a reservation at a romantic Italian restaurant tonight at 7 PM",
                "Hey Siri. Show me the reviews for Alexander’s Steakhouse in Cupertino",
                "Hey Siri. Turn off my alarm",
                "Hey Siri. Delete all alarms",
                "Hey Siri. Turn off my Good Morning alarm",
                "Hey Siri. Show me my alarms",
                "Hey Siri. Is Ian McKellen still alive?",
                "Hey Siri. How tall is Ian McKellen?",
                "Hey Siri. Where was Ian McKellen born?",
                "Hey Siri. Who is Ian McKellen married to?",
                "Hey Siri. Who wrote Harry Potter?",
                "Hey Siri. Who invented the iPhone?",
                "Hey Siri. How far away is the moon?",
                "Hey Siri. How high is Mount Everest?",
                "Hey Siri. What is the population of Switzerland?",
                "Hey Siri. How many calories in a bagel?",
                "Hey Siri. How long do dogs live?",
                "Hey Siri. How many teeth does a dog have?",
                "Hey Siri. What type of Pokémon is Pikachu?",
                "Hey Siri. Spell necessary",
                "Hey Siri. What’s the weather like?",
                "Hey Siri. Do I need an umbrella for tomorrow?",
                "Hey Siri. What’s the weather going to be like in Madrid tomorrow?",
                "Hey Siri. Is there is a chance of rain tomorrow?",
                "Hey Siri. What’s the perceived temperature outside?",
                "Hey Siri. What’s the dew point outside?",
                "Hey Siri. Is it windy outside?",
                "Hey Siri. What’s the pressure outside?",
                "Hey Siri. What’s the visibility outside?",
                "Hey Siri. What is the KP Index?",
                "Hey Siri. How humid is it outside?",
                "Hey Siri. When is the sunrise?",
                "Hey Siri. When is the sunset tomorrow?",
                "Hey Siri. When is the sunrise on Friday?",
                "Hey Siri. When is the sunset in New York?",
                "Hey Siri. What’s the Apple stock price?",
                "Hey Siri. Compare Apple with Alphabet",
                "Hey Siri. Define airplane",
                "Hey Siri. What is the definition of airplane?",
                "Hey Siri. What does the French word maison mean in English?",
                "Hey Siri. Find books by Charles Dickens",
                "Hey Siri. Find movies by Christopher Nolan",
                "Hey Siri. What is the movie Indiana Jones about?",
                "Hey Siri. When was Indiana Jones released?",
                "Hey Siri. Runtime of Indiana Jones?",
                "Hey Siri. Who acted in Indiana Jones?",
                "Hey Siri. Movies with Scarlett Johansson",
                "Hey Siri. Best thriller movies?",
                "Hey Siri. Which movie won Best Picture in 1966?",
                "Hey Siri. What movies are playing this evening?",
                "Hey Siri. Buy three tickets to see The Lego Movie tonight in Sacramento",
                "Hey Siri. Find some movie theaters near my home",
                "Hey Siri. Shuffle my gym playlist",
                "Hey Siri. What’s this song?",
                "Hey Siri. Who sings this?",
                "Hey Siri. I like this song",
                "Hey Siri. What is the point spread in the NFL game?",
                "Hey Siri. How is Chelsea doing?",
                "Hey Siri. Results from Liverpool last game?",
                "Hey Siri. Who’s going to win the Vikings game?",
                "Hey Siri. When is the next Liverpool game?",
                "Hey Siri. What Channel is the Royals game on?",
                "Hey Siri. When is the Super Bowl?",
                "Hey Siri. Flip a coin",
                "Hey Siri. Pick a card",
                "Hey Siri. Roll a twenty-sided die",
                "Hey Siri. Random number between 30 and 60",
                "Hey Siri. See you on the seventh",
                "Hey Siri. What is 1 million divided by 0?",
                "Hey Siri. What is 0 divided by 0?",
                "Hey Siri. What is infinity times infinity?",
                "Hey Siri. Rock paper scissors",
                "Hey Siri. Sudo make me a sandwich",
                "Hey Siri. Tell me a joke",
                "Hey Siri. Tell haiku",
                "Hey Siri. Tell me a tongue twister",
                "Hey Siri. Tell me a story",
                "Hey Siri. Tell me a poem",
                "Hey Siri. Tell me a secret",
                "Hey Siri. Tell me a bedtime story",
                "Hey Siri. Sing me a lullaby",
                "Hey Siri. Beam me up",
                "Hey Siri. Guess what",
                "Hey Siri. Who’s on first?",
                "Hey Siri. Open the pod bay doors",
                "Hey Siri. Sing me a song now",
                "Hey Siri. When is your birthday?",
                "Hey Siri. What’s your sign?",
                "Hey Siri. What’s your favourite animal?",
                "Hey Siri. What color is your hair?",
                "Hey Siri. How much do you weigh?",
                "Hey Siri. Are you smart?",
                "Hey Siri. Are you perfect?",
                "Hey Siri. Do you think I look fat in this?",
                "Hey Siri. Will you marry me?",
                "Hey Siri. May the force be with you",
                "Hey Siri. Can I call you Jarvis?",
                "Hey Siri. When do you sleep?",
                "Hey Siri. How is it to be you?",
                "Hey Siri. Have you seen Star Wars?",
                "Hey Siri. What is your favourite colour?",
                "Hey Siri. What are you going to be for Halloween?",
                "Hey Siri. Do you know pick up lines?",
                "Hey Siri. Mirror mirror on the wall, who’s the fairest of them all?",
                "Hey Siri. What does the fox say?",
                "Hey Siri. Who let the dogs out?",
                "Hey Siri. How much wood could a woodchuck chuck if a woodchuck could chuck wood?",
                "Hey Siri. What is the airspeed velocity of an unladen swallow?",
                "Hey Siri. Why are fire trucks red?",
                "Hey Siri. Why did the chicken cross the road?",
                "Hey Siri. What is the meaning of life?",
                "Hey Siri. When is the end of the world?",
                "Hey Siri. What’s the best phone?",
                "Hey Siri. Can I borrow some money?",
                "Hey Siri. supercalifragilisticexpialidocious",
                "Hey Siri. Rap Beatbox",
                "Hey Siri. Can I call you Cortana?",
                "Hey Siri. You’re the best",
                "Hey Siri. Meow",
                "Hey Siri. I’m sleepy",
                "Hey Siri. How many languages do you speak?",
            ],
            DatasetSize::Small => vec![
                "Hey Siri. What is the factorial of 6?", // mathematics
                "Hey Siri. What day was 90 days ago?",   // time
                "Hey Siri. What is the temperature in living room?", // home
                "Hey Siri. Any missed calls?",           // calls
                "Hey Siri. Read Calendar",               // calendar
                "Hey Siri. Remind me to wash the car",   // reminders
                "Hey Siri. How far is New York from Boston", // maps
                "Hey Siri. How old is Ian McKellen?",    // trivia
                "Hey Siri. What’s the temperature outside?", // weather
                "Hey Siri. Translate car from English to Spanish", // translation
                "Hey Siri. Roll a die",                  // randomness
                "Hey Siri. Is there a God?",             // banter
                "Hey Siri. What’s 2330 dollars in euros?", // conversion
            ],
            DatasetSize::Binary => vec!["Hey Siri. Call John Doe", "Hey Siri. Call Mary Poppins"],
        }
    }
}
