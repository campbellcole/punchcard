// Copyright (C) 2023 Campbell M. Cole
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use async_openai::{
    error::OpenAIError,
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use chrono::{prelude::*, Days};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NlpError {
    #[error("OpenAI error: {0}")]
    OpenAiError(#[from] OpenAIError),
}

fn generate_prompt<T>(datetime: DateTime<T>, nlp: &str) -> String
where
    T: TimeZone,
    T::Offset: std::fmt::Display,
{
    format!(
        "The current date and time is {}.\nInterpret the following natural language then generate an RFC3339 timestamp in the UTC timezone:\n{}",
        datetime.format("%B %d, %Y at %H:%M:%S %Z"),
        nlp,
    )
}

#[allow(unreachable_code, unused_variables)]
pub async fn parse_nlp_timestamp(timestamp: &str) -> Result<DateTime<Local>, NlpError> {
    todo!("NLP is not yet implemented. Waiting for an OpenAI API key.");

    let client = Client::new();

    let now = Utc::now();
    let yesterday_at_noon = Utc::now()
        .checked_sub_days(Days::new(1))
        .unwrap()
        .with_hour(12)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();

    let request = CreateChatCompletionRequestArgs::default()
        // an RFC3339 timestamp in UTC is 23 characters
        .max_tokens(23u16)
        .model("gpt-3.5-turbo")
        .messages([
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(
                    "You are a program which interprets natural language into RFC3339 timestamps.",
                )
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(generate_prompt(now, "yesterday at noon"))
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::Assistant)
                .content(yesterday_at_noon.to_rfc3339())
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(generate_prompt(Local::now(), timestamp))
                .build()?,
        ])
        .build()?;

    let response = client.chat().create(request).await?;

    println!("{:#?}", response);
    for choice in response.choices {
        println!(
            "{}: Role: {}  Content: {}",
            choice.index, choice.message.role, choice.message.content
        );
    }

    Ok(Local::now())
}
