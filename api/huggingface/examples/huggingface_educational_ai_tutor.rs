#![ allow( clippy::pedantic ) ]
#![allow(clippy::missing_inline_in_public_items)]

//! Educational AI Tutor Example
//!
//! This example demonstrates an intelligent tutoring system that provides personalized learning assistance,
//! concept explanations, and adaptive content generation using HuggingFace's inference API.
//!
//! The tutor supports multiple learning styles, complexity levels, subject areas, and includes:
//! - Adaptive concept explanation generation
//! - Intelligent student question answering with context
//! - Learning progress tracking and personalization
//! - Multi-modal explanations using text generation
//! - Assessment generation and automated feedback
//! - Learning path recommendations and guidance
//! - Interactive CLI interface for tutoring sessions
//!
//! # Features
//!
//! - **Multiple Complexity Levels**: Elementary, Middle School, High School, University, Graduate
//! - **Learning Style Adaptation**: Visual, Auditory, Kinesthetic, Reading/Writing, Multimodal
//! - **Subject Coverage**: Mathematics, Science, History, Language Arts, Computer Science, Foreign Language, Arts, General
//! - **Progress Tracking**: Individual student proficiency monitoring and improvement tracking
//! - **Assessment System**: Automated question generation and evaluation with detailed feedback
//! - **Platform Statistics**: Usage analytics and teaching effectiveness metrics
//!
//! # Usage
//!
//! Run the example with your HuggingFace API key:
//!
//! ```bash
//! export HUGGINGFACE_API_KEY="your_api_key_here"
//! cargo run --example educational_ai_tutor --features="full"
//! ```
//!
//! The program presents an interactive CLI with the following commands:
//! - `/explain < concept >` - Generate detailed concept explanations
//! - `/ask < question >` - Ask questions and receive tutoring responses  
//! - `/student < action >` - Student management (register, switch, profile)
//! - `/assess < subject >` - Generate and take assessments
//! - `/progress` - View learning progress and statistics
//! - `/config` - Platform configuration and preferences
//! - `/help` - Show detailed command help
//! - `/quit` - Exit the tutor
//!
//! # Examples
//!
//! - Explain photosynthesis for middle school visual learners
//! - Answer math questions with step-by-step solutions
//! - Generate assessments tailored to student complexity level
//! - Track learning progress across multiple subjects
//! - Adapt teaching style to individual learning preferences

use api_huggingface::
{
  Client,
  environment::HuggingFaceEnvironmentImpl,
  components::
  {
  input::InferenceParameters,
  },
  secret::Secret,
};

use std::
{
  collections::HashMap,
  time::Instant,
  io::{ self, Write },
};
use serde::{ Serialize, Deserialize };

/// Learning complexity levels for content adaptation
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub enum ComplexityLevel
{
  /// Elementary level (grades K-5)
  Elementary,
  /// Middle school level (grades 6-8)
  MiddleSchool,
  /// High school level (grades 9-12)
  HighSchool,
  /// University undergraduate level
  University,
  /// Graduate and advanced level
  Graduate,
}

impl ComplexityLevel
{
  /// Get complexity level name
  pub fn name( &self ) -> &'static str
  {
  match self
  {
      ComplexityLevel::Elementary => "Elementary",
      ComplexityLevel::MiddleSchool => "Middle School",
      ComplexityLevel::HighSchool => "High School",
      ComplexityLevel::University => "University",
      ComplexityLevel::Graduate => "Graduate",
  }
  }

  /// Get age range for this complexity level
  pub fn age_range( &self ) -> ( u8, u8 )
  {
  match self
  {
      ComplexityLevel::Elementary => ( 5, 11 ),
      ComplexityLevel::MiddleSchool => ( 11, 14 ),
      ComplexityLevel::HighSchool => ( 14, 18 ),
      ComplexityLevel::University => ( 18, 22 ),
      ComplexityLevel::Graduate => ( 22, 99 ),
  }
  }

  /// Get vocabulary complexity score (1-10)
  pub fn vocabulary_complexity( &self ) -> u8
  {
  match self
  {
      ComplexityLevel::Elementary => 2,
      ComplexityLevel::MiddleSchool => 4,
      ComplexityLevel::HighSchool => 6,
      ComplexityLevel::University => 8,
      ComplexityLevel::Graduate => 10,
  }
  }

  /// Get recommended explanation length (words)
  pub fn explanation_length_range( &self ) -> ( usize, usize )
  {
  match self
  {
      ComplexityLevel::Elementary => ( 50, 150 ),
      ComplexityLevel::MiddleSchool => ( 100, 250 ),
      ComplexityLevel::HighSchool => ( 150, 350 ),
      ComplexityLevel::University => ( 200, 500 ),
      ComplexityLevel::Graduate => ( 300, 800 ),
  }
  }

  /// Get preferred model for this complexity level
  pub fn preferred_model( &self ) -> &'static str
  {
  // Use Kimi-K2 model for all complexity levels (new Router API)
  "moonshotai/Kimi-K2-Instruct-0905:groq"
  }
}

/// Learning styles for personalized content generation
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub enum LearningStyle
{
  /// Visual learners prefer diagrams and visual aids
  Visual,
  /// Auditory learners prefer verbal explanations
  Auditory,
  /// Kinesthetic learners prefer hands-on activities
  Kinesthetic,
  /// Reading/writing learners prefer text-based content
  ReadingWriting,
  /// Multimodal learners benefit from mixed approaches
  Multimodal,
}

impl LearningStyle
{
  /// Get learning style name
  pub fn name( &self ) -> &'static str
  {
  match self
  {
      LearningStyle::Visual => "Visual",
      LearningStyle::Auditory => "Auditory",
      LearningStyle::Kinesthetic => "Kinesthetic",
      LearningStyle::ReadingWriting => "Reading/Writing",
      LearningStyle::Multimodal => "Multimodal",
  }
  }

  /// Get recommended content format preferences
  pub fn content_preferences( &self ) -> Vec< &'static str >
  {
  match self
  {
      LearningStyle::Visual => vec![ "diagrams", "charts", "mind_maps", "infographics" ],
      LearningStyle::Auditory => vec![ "verbal_explanations", "discussions", "recordings", "mnemonics" ],
      LearningStyle::Kinesthetic => vec![ "hands_on_activities", "experiments", "simulations", "practice_problems" ],
      LearningStyle::ReadingWriting => vec![ "detailed_text", "note_taking", "summaries", "written_exercises" ],
      LearningStyle::Multimodal => vec![ "combined_approaches", "varied_formats", "interactive_content", "adaptive_materials" ],
  }
  }

  /// Get engagement strategies for this learning style
  pub fn engagement_strategies( &self ) -> Vec< &'static str >
  {
  match self
  {
      LearningStyle::Visual => vec![ "use_visual_metaphors", "create_concept_maps", "show_step_by_step_visuals" ],
      LearningStyle::Auditory => vec![ "explain_aloud", "use_analogies", "encourage_discussion" ],
      LearningStyle::Kinesthetic => vec![ "provide_examples", "suggest_practice", "break_into_steps" ],
      LearningStyle::ReadingWriting => vec![ "provide_detailed_notes", "suggest_writing_exercises", "offer_additional_reading" ],
      LearningStyle::Multimodal => vec![ "combine_multiple_methods", "provide_alternatives", "adapt_to_context" ],
  }
  }
}

/// Subject areas for educational content
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub enum Subject
{
  /// Mathematics and quantitative reasoning
  Mathematics,
  /// Physical sciences (physics, chemistry)
  Science,
  /// History and social studies
  History,
  /// Language arts and literature
  LanguageArts,
  /// Computer science and programming
  ComputerScience,
  /// Foreign language learning
  ForeignLanguage,
  /// Arts and creative subjects
  Arts,
  /// General knowledge and interdisciplinary topics
  General,
}

impl Subject
{
  /// Get subject name
  pub fn name( &self ) -> &'static str
  {
  match self
  {
      Subject::Mathematics => "Mathematics",
      Subject::Science => "Science",
      Subject::History => "History",
      Subject::LanguageArts => "Language Arts",
      Subject::ComputerScience => "Computer Science",
      Subject::ForeignLanguage => "Foreign Language",
      Subject::Arts => "Arts",
      Subject::General => "General Knowledge",
  }
  }

  /// Get typical teaching approaches for this subject
  pub fn teaching_approaches( &self ) -> Vec< &'static str >
  {
  match self
  {
      Subject::Mathematics => vec![ "step_by_step_solutions", "problem_solving", "formula_derivation", "visual_proofs" ],
      Subject::Science => vec![ "conceptual_explanations", "real_world_applications", "experimental_design", "cause_and_effect" ],
      Subject::History => vec![ "chronological_narrative", "cause_and_effect", "primary_sources", "contextual_analysis" ],
      Subject::LanguageArts => vec![ "textual_analysis", "writing_techniques", "grammar_explanation", "literary_devices" ],
      Subject::ComputerScience => vec![ "code_examples", "algorithm_explanation", "debugging_techniques", "system_design" ],
      Subject::ForeignLanguage => vec![ "conversation_practice", "grammar_rules", "cultural_context", "pronunciation_guides" ],
      Subject::Arts => vec![ "technique_demonstration", "historical_context", "creative_inspiration", "skill_building" ],
      Subject::General => vec![ "interdisciplinary_connections", "critical_thinking", "research_methods", "synthesis" ],
  }
  }
}

/// Student profile for personalized learning
#[ derive( Debug, Clone ) ]
pub struct StudentProfile
{
  /// Student's unique identifier
  pub student_id : String,
  /// Student's name
  pub name : String,
  /// Current complexity level
  pub complexity_level : ComplexityLevel,
  /// Preferred learning style
  pub learning_style : LearningStyle,
  /// Strong subject areas
  pub strong_subjects : Vec< Subject >,
  /// Areas needing improvement
  pub weak_subjects : Vec< Subject >,
  /// Learning goals and objectives
  pub learning_goals : Vec< String >,
  /// Learning pace (1.0 = average, >1.0 = faster, <1.0 = slower)
  pub learning_pace : f32,
  /// Additional learning preferences
  pub preferences : HashMap< String, String >,
}

/// Concept explanation request
#[ derive( Debug, Clone ) ]
pub struct ConceptRequest
{
  /// The concept to explain
  pub concept : String,
  /// Subject area
  pub subject : Subject,
  /// Target complexity level
  pub complexity_level : ComplexityLevel,
  /// Student's learning style
  pub learning_style : LearningStyle,
  /// Additional context or background
  pub context : Option< String >,
  /// Prerequisites that student should know
  pub prerequisites : Vec< String >,
  /// Specific aspects to focus on
  pub focus_areas : Vec< String >,
}

/// Generated concept explanation
#[ derive( Debug, Clone ) ]
pub struct ConceptExplanation
{
  /// The concept that was explained
  pub concept : String,
  /// Generated explanation text
  pub explanation : String,
  /// Key points and takeaways
  pub key_points : Vec< String >,
  /// Examples and analogies used
  pub examples : Vec< String >,
  /// Related concepts to explore
  pub related_concepts : Vec< String >,
  /// Suggested practice activities
  pub practice_suggestions : Vec< String >,
  /// Pedagogical effectiveness score (0.0-1.0)
  pub effectiveness_score : f32,
  /// Complexity appropriateness score (0.0-1.0)
  pub complexity_score : f32,
  /// Generation time in milliseconds
  pub generation_time_ms : u64,
}

/// Student question for the tutor
#[ derive( Debug, Clone ) ]
pub struct StudentQuestion
{
  /// Question ID
  pub question_id : String,
  /// The student's question
  pub question_text : String,
  /// Subject area (if identifiable)
  pub subject : Option< Subject >,
  /// Complexity level of the question
  pub complexity_level : ComplexityLevel,
  /// Context or background information
  pub context : Option< String >,
  /// Student's current understanding level
  pub understanding_level : Option< f32 >,
}

/// Tutor's response to student question
#[ derive( Debug, Clone ) ]
pub struct TutorResponse
{
  /// Response to the question
  pub answer : String,
  /// Confidence in the answer (0.0-1.0)
  pub confidence : f32,
  /// Educational value score (0.0-1.0)
  pub educational_value : f32,
  /// Whether follow-up questions are suggested
  pub suggests_follow_up : bool,
  /// Follow-up questions or exercises
  pub follow_up_suggestions : Vec< String >,
  /// Assessment of student's understanding
  pub understanding_assessment : f32,
  /// Response generation time
  pub response_time_ms : u64,
}

/// Learning progress tracking
#[ derive( Debug, Clone ) ]
pub struct LearningProgress
{
  /// Subject being tracked
  pub subject : Subject,
  /// Current proficiency level (0.0-1.0)
  pub proficiency_level : f32,
  /// Improvement rate (change per session)
  pub improvement_rate : f32,
  /// Number of concepts mastered
  pub concepts_mastered : u32,
  /// Number of questions answered correctly
  pub correct_answers : u32,
  /// Total number of questions attempted
  pub total_questions : u32,
  /// Average response time (seconds)
  pub average_response_time : f32,
  /// Last updated timestamp
  pub last_updated : String,
}

/// Assessment and feedback generation
#[ derive( Debug, Clone ) ]
pub struct Assessment
{
  /// Assessment ID
  pub assessment_id : String,
  /// Subject area
  pub subject : Subject,
  /// Questions in the assessment
  pub questions : Vec< AssessmentQuestion >,
  /// Total points possible
  pub total_points : u32,
  /// Time limit in minutes
  pub time_limit_minutes : Option< u32 >,
  /// Difficulty level
  pub difficulty_level : ComplexityLevel,
}

/// Individual assessment question
#[ derive( Debug, Clone ) ]
pub struct AssessmentQuestion
{
  /// Question ID
  pub question_id : String,
  /// Question text
  pub question : String,
  /// Question type
  pub question_type : QuestionType,
  /// Correct answer(s)
  pub correct_answers : Vec< String >,
  /// Points value
  pub points : u32,
  /// Explanation for the correct answer
  pub explanation : String,
}

/// Types of assessment questions
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
pub enum QuestionType
{
  /// Multiple choice question
  MultipleChoice,
  /// True/false question
  TrueFalse,
  /// Short answer question
  ShortAnswer,
  /// Essay question
  Essay,
  /// Fill in the blank
  FillInTheBlank,
  /// Mathematical problem
  Mathematical,
}

/// Assessment results and feedback
#[ derive( Debug, Clone ) ]
pub struct AssessmentResult
{
  /// Student's responses
  pub responses : Vec< String >,
  /// Score achieved
  pub score : u32,
  /// Percentage score
  pub percentage : f32,
  /// Time taken in minutes
  pub time_taken : f32,
  /// Detailed feedback for each question
  pub question_feedback : Vec< QuestionFeedback >,
  /// Overall performance assessment
  pub performance_summary : String,
  /// Areas for improvement
  pub improvement_areas : Vec< String >,
  /// Recommended next steps
  pub recommendations : Vec< String >,
}

/// Feedback for individual question
#[ derive( Debug, Clone ) ]
pub struct QuestionFeedback
{
  /// Question ID
  pub question_id : String,
  /// Whether the answer was correct
  pub is_correct : bool,
  /// Points earned for this question
  pub points_earned : u32,
  /// Feedback message
  pub feedback : String,
  /// Hints for improvement
  pub improvement_hints : Vec< String >,
}

/// AI Tutor platform
#[ derive( Debug, Clone ) ]
pub struct AITutorPlatform
{
  /// HuggingFace API client
  client : Client< HuggingFaceEnvironmentImpl >,
  /// Registered student profiles
  students : HashMap< String, StudentProfile >,
  /// Learning progress tracking
  progress_tracking : HashMap< String, HashMap< Subject, LearningProgress > >,
  /// Platform configuration
  config : TutorConfig,
  /// Teaching statistics
  statistics : TutorStatistics,
  /// Currently active student
  current_student : Option< String >,
}

/// Tutor platform configuration
#[ derive( Debug, Clone ) ]
pub struct TutorConfig
{
  /// Default model for content generation
  pub default_model : String,
  /// Maximum explanation length
  pub max_explanation_length : usize,
  /// Enable adaptive difficulty adjustment
  pub enable_adaptive_difficulty : bool,
  /// Progress tracking frequency
  pub progress_update_frequency : u32,
  /// Default learning style for new students
  pub default_learning_style : LearningStyle,
  /// Assessment frequency (sessions between assessments)
  pub assessment_frequency : u32,
}

impl Default for TutorConfig
{
  fn default() -> Self
  {
  Self
  {
      default_model : "moonshotai/Kimi-K2-Instruct-0905:groq".to_string(),
      max_explanation_length : 1000,
      enable_adaptive_difficulty : true,
      progress_update_frequency : 5,
      default_learning_style : LearningStyle::Multimodal,
      assessment_frequency : 10,
  }
  }
}

/// Platform teaching statistics
#[ derive( Debug, Clone, Default ) ]
pub struct TutorStatistics
{
  /// Total number of explanations generated
  pub total_explanations : u64,
  /// Total questions answered
  pub total_questions_answered : u64,
  /// Average explanation effectiveness
  pub average_effectiveness : f32,
  /// Total learning time (minutes)
  pub total_learning_time : u64,
  /// Students served
  pub students_served : u32,
  /// Subject popularity
  pub subject_popularity : HashMap< Subject, u32 >,
  /// Complexity level distribution
  pub complexity_distribution : HashMap< ComplexityLevel, u32 >,
}

impl AITutorPlatform
{
  /// Create a new AI tutor platform
  pub fn new( client : Client< HuggingFaceEnvironmentImpl > ) -> Self
  {
  Self
  {
      client,
      students : HashMap::new(),
      progress_tracking : HashMap::new(),
      config : TutorConfig::default(),
      statistics : TutorStatistics::default(),
      current_student : None,
  }
  }

  /// Create platform with custom configuration
  pub fn with_config( client : Client< HuggingFaceEnvironmentImpl >, config : TutorConfig ) -> Self
  {
  Self
  {
      client,
      students : HashMap::new(),
      progress_tracking : HashMap::new(),
      config,
      statistics : TutorStatistics::default(),
      current_student : None,
  }
  }

  /// Register a new student
  pub fn register_student( &mut self, profile : StudentProfile )
  {
  let student_id = profile.student_id.clone();
  self.students.insert( student_id.clone(), profile );

  // Initialize progress tracking
  let mut progress = HashMap::new();
  for subject in [ Subject::Mathematics, Subject::Science, Subject::History, Subject::LanguageArts,
                     Subject::ComputerScience, Subject::ForeignLanguage, Subject::Arts, Subject::General ]
  {
      progress.insert( subject, LearningProgress
      {
  subject,
  proficiency_level : 0.5, // Start at average
  improvement_rate : 0.0,
  concepts_mastered : 0,
  correct_answers : 0,
  total_questions : 0,
  average_response_time : 0.0,
  last_updated : chrono::Utc::now().format( "%Y-%m-%d" ).to_string(),
      } );
  }
  self.progress_tracking.insert( student_id, progress );
  self.statistics.students_served += 1;
  }

  /// Set current active student
  pub fn set_current_student( &mut self, student_id : Option< String > )
  {
  self.current_student = student_id;
  }

  /// Get current student profile
  pub fn get_current_student( &self ) -> Option< &StudentProfile >
  {
  self.current_student.as_ref().and_then( | id | self.students.get( id ) )
  }

  /// Generate concept explanation
  pub async fn explain_concept( &mut self, request : &ConceptRequest ) -> Result< ConceptExplanation, Box< dyn std::error::Error > >
  {
  let start_time = Instant::now();

  // Build explanation prompt
  let prompt = self.build_explanation_prompt( request )?;

  // Select appropriate model
  let model = request.complexity_level.preferred_model();

  // Set generation parameters
  let params = InferenceParameters::new()
      .with_max_new_tokens( self.calculate_max_tokens( request.complexity_level ) )
      .with_temperature( 0.7 ) // Balanced creativity and consistency
      .with_top_p( 0.9 );

  // Generate explanation
  let response = self.client.inference().create_with_parameters( &prompt, model, params ).await?;

  let explanation_text = response.extract_text_or_default( "Unable to generate explanation." );

  let generation_time = start_time.elapsed().as_millis() as u64;

  // Process explanation and extract components
  let explanation = self.process_explanation( &explanation_text, request, generation_time )?;

  // Update statistics
  self.update_explanation_statistics( &request.subject, request.complexity_level );

  Ok( explanation )
  }

  /// Answer student question
  pub async fn answer_question( &mut self, question : &StudentQuestion, student_id : Option< &str > ) -> Result< TutorResponse, Box< dyn std::error::Error > >
  {
  let start_time = Instant::now();

  // Build question answering prompt
  let prompt = self.build_question_prompt( question, student_id )?;

  // Select model based on complexity
  let model = question.complexity_level.preferred_model();

  // Set parameters for question answering
  let params = InferenceParameters::new()
      .with_max_new_tokens( 300 )
      .with_temperature( 0.5 ) // Lower temperature for more focused answers
      .with_top_p( 0.8 );

  // Generate response
  let response = self.client.inference().create_with_parameters( &prompt, model, params ).await?;

  let answer_text = response.extract_text_or_default( "I'm not sure how to answer that question." );

  let response_time = start_time.elapsed().as_millis() as u64;

  // Process answer and generate response
  let tutor_response = self.process_answer( &answer_text, question, response_time )?;

  // Update progress if student ID provided
  if let Some( id ) = student_id
  {
      self.update_student_progress( id, question, &tutor_response );
  }

  // Update statistics
  self.statistics.total_questions_answered += 1;

  Ok( tutor_response )
  }

  /// Generate assessment for student
  pub fn generate_assessment( &self, subject : Subject, complexity_level : ComplexityLevel, num_questions : usize ) -> Assessment
  {
  let mut questions = Vec::new();

  // Generate different types of questions
  for i in 0..num_questions
  {
      let question_type = match i % 6
      {
  0 => QuestionType::MultipleChoice,
  1 => QuestionType::TrueFalse,
  2 => QuestionType::ShortAnswer,
  3 => QuestionType::FillInTheBlank,
  4 => QuestionType::Mathematical,
  _ => QuestionType::Essay,
      };

      let question_text = Self::generate_sample_question( subject, complexity_level, question_type );
      let correct_answer = Self::generate_sample_answer( question_type );

      questions.push( AssessmentQuestion
      {
  question_id : format!( "q_{}", i + 1 ),
  question : question_text,
  question_type,
  correct_answers : vec![ correct_answer ],
  points : if question_type == QuestionType::Essay { 20 } else { 10 },
  explanation : "This is a sample explanation for the correct answer.".to_string(),
      } );
  }

  let total_points = questions.iter().map( | q | q.points ).sum();

  Assessment
  {
      assessment_id : format!( "assessment_{}_{:?}_{}", subject.name(), complexity_level, chrono::Utc::now().timestamp() ),
      subject,
      questions,
      total_points,
      time_limit_minutes : Some( num_questions as u32 * 3 ), // 3 minutes per question
      difficulty_level : complexity_level,
  }
  }

  /// Evaluate assessment responses
  pub fn evaluate_assessment( &self, assessment : &Assessment, responses : Vec< String > ) -> AssessmentResult
  {
  let mut score = 0;
  let mut question_feedback = Vec::new();
  let time_taken = 15.5; // Simulated time

  for ( i, response ) in responses.iter().enumerate()
  {
      if let Some( question ) = assessment.questions.get( i )
      {
  let is_correct = Self::evaluate_response( response, &question.correct_answers, question.question_type );
  let points_earned = if is_correct { question.points } else { 0 };
  score += points_earned;

  question_feedback.push( QuestionFeedback
  {
          question_id : question.question_id.clone(),
          is_correct,
          points_earned,
          feedback : if is_correct
          {
      "Correct! Well done.".to_string()
          }
          else
          {
      format!( "Incorrect. The correct answer is : {}", question.correct_answers[ 0 ] )
          },
          improvement_hints : if is_correct
          {
      Vec::new()
          }
          else
          {
      vec![ "Review the concept explanation and try practice problems.".to_string() ]
          },
  } );
      }
  }

  let percentage = if assessment.total_points > 0 { ( score as f32 / assessment.total_points as f32 ) * 100.0 } else { 0.0 };

  AssessmentResult
  {
      responses,
      score,
      percentage,
      time_taken,
      question_feedback,
      performance_summary : Self::get_performance_summary( percentage ),
      improvement_areas : vec![ "Practice more problems".to_string(), "Review key concepts".to_string() ],
      recommendations : vec![ "Focus on fundamentals".to_string(), "Seek additional help if needed".to_string() ],
  }
  }

  /// Get student progress
  pub fn get_student_progress( &self, student_id : &str ) -> Option< &HashMap< Subject, LearningProgress > >
  {
  self.progress_tracking.get( student_id )
  }

  /// Get platform statistics
  pub fn get_statistics( &self ) -> &TutorStatistics
  {
  &self.statistics
  }

  /// Get all registered students
  pub fn get_students( &self ) -> &HashMap< String, StudentProfile >
  {
  &self.students
  }

  /// Build explanation prompt
  fn build_explanation_prompt( &self, request : &ConceptRequest ) -> Result< String, Box< dyn std::error::Error > >
  {
  let mut prompt = format!(
      "Explain the concept of '{}' in {} for {} level students.\n",
      request.concept,
      request.subject.name(),
      request.complexity_level.name()
  );

  // Add learning style considerations
  let strategies = request.learning_style.engagement_strategies();
  prompt.push_str( &format!( "Use {} teaching strategies. ", strategies.join( ", " ) ) );

  // Add context if provided
  if let Some( ref context ) = request.context
  {
      prompt.push_str( &format!( "Context : {}. ", context ) );
  }

  // Add prerequisites
  if !request.prerequisites.is_empty()
  {
      prompt.push_str( &format!( "Assume knowledge of : {}. ", request.prerequisites.join( ", " ) ) );
  }

  prompt.push_str( "\n\nExplanation:" );

  Ok( prompt )
  }

  /// Build question answering prompt
  fn build_question_prompt( &self, question : &StudentQuestion, student_id : Option< &str > ) -> Result< String, Box< dyn std::error::Error > >
  {
  let mut prompt = format!(
      "A {} level student asks : '{}'\n",
      question.complexity_level.name(),
      question.question_text
  );

  // Add student context if available
  if let Some( id ) = student_id
  {
      if let Some( profile ) = self.students.get( id )
      {
  prompt.push_str( &format!( "Student learning style : {}. ", profile.learning_style.name() ) );
      }
  }

  // Add subject context if identified
  if let Some( subject ) = question.subject
  {
      prompt.push_str( &format!( "Subject area : {}. ", subject.name() ) );
  }

  prompt.push_str( "\n\nProvide a helpful, educational answer:" );

  Ok( prompt )
  }

  /// Calculate max tokens based on complexity level
  fn calculate_max_tokens( &self, complexity_level : ComplexityLevel ) -> u32
  {
  let ( _min_words, max_words ) = complexity_level.explanation_length_range();
  ( max_words as f32 * 1.3 ) as u32 // Estimate tokens as 1.3x words
  }

  /// Process explanation response
  fn process_explanation( &self, text : &str, request : &ConceptRequest, generation_time : u64 ) -> Result< ConceptExplanation, Box< dyn std::error::Error > >
  {
  let word_count = text.split_whitespace().count();
  let ( min_words, max_words ) = request.complexity_level.explanation_length_range();

  // Calculate scores
  let complexity_score = if word_count >= min_words && word_count <= max_words { 1.0 }
                          else if word_count < min_words { 0.7 }
                          else { 0.8 };

  let effectiveness_score = 0.8; // Simplified scoring

  // Extract key components (simplified)
  let key_points = vec![
      "Main concept definition".to_string(),
      "Key characteristics".to_string(),
      "Important relationships".to_string()
  ];

  let examples = vec![
      "Practical example 1".to_string(),
      "Analogy or metaphor".to_string()
  ];

  let related_concepts = vec![
      "Related concept A".to_string(),
      "Related concept B".to_string()
  ];

  let practice_suggestions = vec![
      "Practice problem 1".to_string(),
      "Hands-on activity".to_string()
  ];

  Ok( ConceptExplanation
  {
      concept : request.concept.clone(),
      explanation : text.trim().to_string(),
      key_points,
      examples,
      related_concepts,
      practice_suggestions,
      effectiveness_score,
      complexity_score,
      generation_time_ms : generation_time,
  } )
  }

  /// Process answer response
  fn process_answer( &self, text : &str, _question : &StudentQuestion, response_time : u64 ) -> Result< TutorResponse, Box< dyn std::error::Error > >
  {
  let confidence = if text.len() > 50 { 0.8 } else { 0.6 };
  let educational_value = 0.75; // Simplified scoring

  let follow_up_suggestions = vec![
      "Would you like me to explain this concept in more detail?".to_string(),
      "Do you have any related questions?".to_string(),
  ];

  Ok( TutorResponse
  {
      answer : text.trim().to_string(),
      confidence,
      educational_value,
      suggests_follow_up : true,
      follow_up_suggestions,
      understanding_assessment : 0.7, // Simplified assessment
      response_time_ms : response_time,
  } )
  }

  /// Update student progress
  fn update_student_progress( &mut self, student_id : &str, question : &StudentQuestion, response : &TutorResponse )
  {
  if let Some( student_progress ) = self.progress_tracking.get_mut( student_id )
  {
      if let Some( subject ) = question.subject
      {
  if let Some( progress ) = student_progress.get_mut( &subject )
  {
          progress.total_questions += 1;
          if response.understanding_assessment > 0.7
          {
      progress.correct_answers += 1;
          }

          // Update proficiency based on performance
          let accuracy = if progress.total_questions > 0 { progress.correct_answers as f32 / progress.total_questions as f32 } else { 0.0 };
          progress.proficiency_level = ( progress.proficiency_level + accuracy ) / 2.0;
          progress.last_updated = chrono::Utc::now().format( "%Y-%m-%d" ).to_string();
  }
      }
  }
  }

  /// Update explanation statistics
  fn update_explanation_statistics( &mut self, subject : &Subject, complexity_level : ComplexityLevel )
  {
  self.statistics.total_explanations += 1;
  *self.statistics.subject_popularity.entry( *subject ).or_insert( 0 ) += 1;
  *self.statistics.complexity_distribution.entry( complexity_level ).or_insert( 0 ) += 1;
  }

  /// Generate sample question for assessment
  fn generate_sample_question( subject : Subject, complexity_level : ComplexityLevel, question_type : QuestionType ) -> String
  {
  match ( subject, complexity_level, question_type )
  {
      ( Subject::Mathematics, ComplexityLevel::Elementary, QuestionType::MultipleChoice ) =>
  "What is 7 + 5? A) 10  B) 12  C) 14  D) 15".to_string(),
      ( Subject::Mathematics, ComplexityLevel::HighSchool, QuestionType::Mathematical ) =>
  "Solve for x : 2x + 3 = 11".to_string(),
      ( Subject::Science, ComplexityLevel::MiddleSchool, QuestionType::TrueFalse ) =>
  "True or False : Plants produce oxygen during photosynthesis.".to_string(),
      ( Subject::History, ComplexityLevel::University, QuestionType::Essay ) =>
  "Analyze the causes and effects of the Industrial Revolution on social structures.".to_string(),
      _ => format!( "Sample {} question for {} level in {}", subject.name(), complexity_level.name(), question_type as u8 ),
  }
  }

  /// Generate sample answer for question type
  fn generate_sample_answer( question_type : QuestionType ) -> String
  {
  match question_type
  {
      QuestionType::MultipleChoice => "B) 12".to_string(),
      QuestionType::TrueFalse => "True".to_string(),
      QuestionType::Mathematical => "x = 4".to_string(),
      QuestionType::ShortAnswer => "Sample short answer".to_string(),
      QuestionType::FillInTheBlank => "photosynthesis".to_string(),
      QuestionType::Essay => "Sample essay answer discussing key points...".to_string(),
  }
  }

  /// Evaluate response against correct answers
  fn evaluate_response( response : &str, correct_answers : &[ String ], question_type : QuestionType ) -> bool
  {
  let response_clean = response.trim().to_lowercase();
  
  match question_type
  {
      QuestionType::Essay => response.len() > 20, // Simplified - just check for substantial content
      _ => correct_answers.iter().any( | correct | correct.trim().to_lowercase() == response_clean ),
  }
  }

  /// Get performance summary based on percentage
  fn get_performance_summary( percentage : f32 ) -> String
  {
  if percentage >= 90.0 { "Excellent performance! Outstanding understanding demonstrated." }
  else if percentage >= 80.0 { "Good work with room for improvement. Solid grasp of concepts." }
  else if percentage >= 70.0 { "Satisfactory performance. Some concepts may need reinforcement." }
  else if percentage >= 60.0 { "Below average performance. Additional study recommended." }
  else { "Needs significant improvement. Consider reviewing fundamental concepts." }.to_string()
  }
}

/// Educational AI Tutor CLI System
#[ derive( Debug ) ]
pub struct EducationalTutorCLI
{
  platform : AITutorPlatform,
  session_active : bool,
}

impl EducationalTutorCLI
{
  /// Create new CLI system
  pub fn new( platform : AITutorPlatform ) -> Self
  {
  Self
  {
      platform,
      session_active : true,
  }
  }

  /// Run the interactive CLI
  pub async fn run( &mut self ) -> Result< (), Box< dyn std::error::Error > >
  {
  self.print_welcome();
  self.register_sample_students();

  while self.session_active
  {
      print!( "🎓 AI Tutor > " );
      io::stdout().flush()?;

      let mut input = String::new();
      io::stdin().read_line( &mut input )?;
      let input = input.trim();

      if input.is_empty()
      {
  continue;
      }

      match self.process_command( input ).await
      {
  Ok( _ ) => {},
  Err( e ) => println!( "❌ Error : {}", e ),
      }
  }

  Ok( () )
  }

  /// Process user commands
  async fn process_command( &mut self, input : &str ) -> Result< (), Box< dyn std::error::Error > >
  {
  let parts : Vec< &str > = input.splitn( 2, ' ' ).collect();
  let command = parts[ 0 ].to_lowercase();

  match command.as_str()
  {
      "/explain" => self.handle_explain_command( parts.get( 1 ).unwrap_or( &"" ) ).await,
      "/ask" => self.handle_ask_command( parts.get( 1 ).unwrap_or( &"" ) ).await,
      "/student" => self.handle_student_command( parts.get( 1 ).unwrap_or( &"" ) ),
      "/assess" => self.handle_assess_command( parts.get( 1 ).unwrap_or( &"" ) ),
      "/progress" => self.handle_progress_command(),
      "/config" => self.handle_config_command(),
      "/help" => self.print_help(),
      "/quit" | "/exit" => { self.session_active = false; Ok( () ) },
      _ => 
      {
  println!( "❌ Unknown command : {}", command );
  println!( "💡 Type /help for available commands" );
  Ok( () )
      }
  }
  }

  /// Handle concept explanation command
  async fn handle_explain_command( &mut self, args : &str ) -> Result< (), Box< dyn std::error::Error > >
  {
  if args.is_empty()
  {
      println!( "❌ Please provide a concept to explain" );
      println!( "💡 Example : /explain photosynthesis" );
      return Ok( () );
  }

  let current_student = self.platform.get_current_student().cloned();
  let ( complexity_level, learning_style ) = if let Some( ref student ) = current_student
  {
      ( student.complexity_level, student.learning_style )
  }
  else
  {
      ( ComplexityLevel::HighSchool, LearningStyle::Multimodal )
  };

  let request = ConceptRequest
  {
      concept : args.to_string(),
      subject : Subject::General, // Could be enhanced to detect subject
      complexity_level,
      learning_style,
      context : Some( "Interactive tutoring session".to_string() ),
      prerequisites : Vec::new(),
      focus_areas : Vec::new(),
  };

  println!( "🔍 Generating explanation for '{}'...", args );
  
  match self.platform.explain_concept( &request ).await
  {
      Ok( explanation ) =>
      {
  self.display_concept_explanation( &explanation );
      }
      Err( e ) =>
      {
  println!( "❌ Failed to generate explanation : {}", e );
      }
  }

  Ok( () )
  }

  /// Handle question command
  async fn handle_ask_command( &mut self, args : &str ) -> Result< (), Box< dyn std::error::Error > >
  {
  if args.is_empty()
  {
      println!( "❌ Please provide a question to ask" );
      println!( "💡 Example : /ask What is the Pythagorean theorem?" );
      return Ok( () );
  }

  let current_student = self.platform.get_current_student().cloned();
  let complexity_level = current_student.as_ref().map( | s | s.complexity_level ).unwrap_or( ComplexityLevel::HighSchool );

  let question = StudentQuestion
  {
      question_id : format!( "cli_q_{}", chrono::Utc::now().timestamp() ),
      question_text : args.to_string(),
      subject : None, // Could be enhanced to detect subject
      complexity_level,
      context : Some( "Interactive CLI session".to_string() ),
      understanding_level : Some( 0.6 ),
  };

  println!( "🤔 Thinking about your question..." );

  let current_student_id = self.platform.current_student.clone();
  match self.platform.answer_question( &question, current_student_id.as_deref() ).await
  {
      Ok( response ) =>
      {
  self.display_tutor_response( &response );
      }
      Err( e ) =>
      {
  println!( "❌ Failed to answer question : {}", e );
      }
  }

  Ok( () )
  }

  /// Handle student management command
  fn handle_student_command( &mut self, args : &str ) -> Result< (), Box< dyn std::error::Error > >
  {
  let parts : Vec< &str > = args.split_whitespace().collect();

  match parts.first().copied().unwrap_or( "" )
  {
      "list" => self.list_students(),
      "switch" => self.switch_student( parts.get( 1 ).copied().unwrap_or( "" ) ),
      "profile" => self.show_student_profile(),
      "register" => self.register_new_student(),
      "" => 
      {
  println!( "📚 Student commands:" );
  println!( "  /student list     - List all registered students" );
  println!( "  /student switch < id > - Switch to different student" );
  println!( "  /student profile  - Show current student profile" );
  println!( "  /student register - Register new student (interactive)" );
      }
      _ => 
      {
  println!( "❌ Unknown student command : {}", parts[ 0 ] );
      }
  }
  
  Ok( () )
  }

  /// Handle assessment command
  fn handle_assess_command( &mut self, args : &str ) -> Result< (), Box< dyn std::error::Error > >
  {
  let subject = match args.to_lowercase().as_str()
  {
      "math" | "mathematics" => Subject::Mathematics,
      "science" => Subject::Science,
      "history" => Subject::History,
      "english" | "language" => Subject::LanguageArts,
      "cs" | "programming" => Subject::ComputerScience,
      "art" | "arts" => Subject::Arts,
      "" => Subject::General,
      _ => 
      {
  println!( "❌ Unknown subject : {}", args );
  println!( "💡 Available subjects : math, science, history, english, cs, art" );
  return Ok( () );
      }
  };

  let complexity_level = self.platform.get_current_student()
      .map( | s | s.complexity_level )
      .unwrap_or( ComplexityLevel::HighSchool );

  let assessment = self.platform.generate_assessment( subject, complexity_level, 5 );
  self.conduct_assessment( &assessment );

  Ok( () )
  }

  /// Handle progress command
  fn handle_progress_command( &mut self ) -> Result< (), Box< dyn std::error::Error > >
  {
  if let Some( current_id ) = &self.platform.current_student.clone()
  {
      if let Some( progress ) = self.platform.get_student_progress( current_id )
      {
  self.display_student_progress( progress );
      }
      else
      {
  println!( "❌ No progress data found for current student" );
      }
  }
  else
  {
      println!( "❌ No student selected. Use /student switch < id > to select a student" );
  }

  // Also show platform statistics
  self.display_platform_statistics();
  Ok( () )
  }

  /// Handle config command
  fn handle_config_command( &mut self ) -> Result< (), Box< dyn std::error::Error > >
  {
  println!( "⚙️  Platform Configuration:" );
  println!( "   Default Model : {}", self.platform.config.default_model );
  println!( "   Max Explanation Length : {} words", self.platform.config.max_explanation_length );
  println!( "   Adaptive Difficulty : {}", if self.platform.config.enable_adaptive_difficulty { "Enabled" } else { "Disabled" } );
  println!( "   Progress Update Frequency : Every {} questions", self.platform.config.progress_update_frequency );
  println!( "   Default Learning Style : {}", self.platform.config.default_learning_style.name() );
  println!( "   Assessment Frequency : Every {} sessions", self.platform.config.assessment_frequency );
  
  Ok( () )
  }

  /// Display concept explanation
  fn display_concept_explanation( &self, explanation : &ConceptExplanation )
  {
  println!( "\n📖 Concept : {}", explanation.concept );
  println!( "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" );
  
  println!( "\n💡 Explanation:" );
  println!( "{}", explanation.explanation );
  
  if !explanation.key_points.is_empty()
  {
      println!( "\n🔑 Key Points:" );
      for ( i, point ) in explanation.key_points.iter().enumerate()
      {
  println!( "   {}. {}", i + 1, point );
      }
  }
  
  if !explanation.examples.is_empty()
  {
      println!( "\n📋 Examples:" );
      for ( i, example ) in explanation.examples.iter().enumerate()
      {
  println!( "   {}. {}", i + 1, example );
      }
  }
  
  if !explanation.practice_suggestions.is_empty()
  {
      println!( "\n🎯 Practice Suggestions:" );
      for ( i, suggestion ) in explanation.practice_suggestions.iter().enumerate()
      {
  println!( "   {}. {}", i + 1, suggestion );
      }
  }
  
  println!( "\n📊 Quality Metrics:" );
  println!( "   Effectiveness : {:.1}%", explanation.effectiveness_score * 100.0 );
  println!( "   Complexity Appropriateness : {:.1}%", explanation.complexity_score * 100.0 );
  println!( "   Generation Time : {}ms", explanation.generation_time_ms );
  
  if !explanation.related_concepts.is_empty()
  {
      println!( "\n🔗 Related Concepts to Explore:" );
      for concept in &explanation.related_concepts
      {
  println!( "   • {}", concept );
      }
  }
  
  println!();
  }

  /// Display tutor response
  fn display_tutor_response( &self, response : &TutorResponse )
  {
  println!( "\n🎓 Tutor Response:" );
  println!( "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" );
  
  println!( "\n💬 Answer:" );
  println!( "{}", response.answer );
  
  if response.suggests_follow_up && !response.follow_up_suggestions.is_empty()
  {
      println!( "\n💭 Follow-up Suggestions:" );
      for ( i, suggestion ) in response.follow_up_suggestions.iter().enumerate()
      {
  println!( "   {}. {}", i + 1, suggestion );
      }
  }
  
  println!( "\n📊 Response Metrics:" );
  println!( "   Confidence : {:.1}%", response.confidence * 100.0 );
  println!( "   Educational Value : {:.1}%", response.educational_value * 100.0 );
  println!( "   Understanding Assessment : {:.1}%", response.understanding_assessment * 100.0 );
  println!( "   Response Time : {}ms", response.response_time_ms );
  println!();
  }

  /// List all students
  fn list_students( &self )
  {
  println!( "\n👥 Registered Students:" );
  
  if self.platform.students.is_empty()
  {
      println!( "   No students registered yet." );
      return;
  }
  
  for ( id, student ) in &self.platform.students
  {
      let current_marker = if Some( id ) == self.platform.current_student.as_ref() { "★ " } else { "  " };
      println!( "{}🎓 {} ({})", current_marker, student.name, id );
      println!( "     Level : {} | Style : {}", student.complexity_level.name(), student.learning_style.name() );
  }
  println!();
  }

  /// Switch current student
  fn switch_student( &mut self, student_id : &str )
  {
  if student_id.is_empty()
  {
      println!( "❌ Please provide student ID" );
      return;
  }
  
  if self.platform.students.contains_key( student_id )
  {
      self.platform.set_current_student( Some( student_id.to_string() ) );
      let student = &self.platform.students[ student_id ];
      println!( "✅ Switched to student : {} ({})", student.name, student_id );
  }
  else
  {
      println!( "❌ Student '{}' not found", student_id );
      println!( "💡 Use '/student list' to see available students" );
  }
  }

  /// Show current student profile
  fn show_student_profile( &self )
  {
  if let Some( student ) = self.platform.get_current_student()
  {
      println!( "\n👤 Current Student Profile:" );
      println!( "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" );
      println!( "   Name : {}", student.name );
      println!( "   ID: {}", student.student_id );
      println!( "   Complexity Level : {}", student.complexity_level.name() );
      println!( "   Learning Style : {}", student.learning_style.name() );
      println!( "   Learning Pace : {:.1}x average", student.learning_pace );
      
      if !student.strong_subjects.is_empty()
      {
  println!( "   Strong Subjects : {}", student.strong_subjects.iter().map( | s | s.name() ).collect::< Vec< _ > >().join( ", " ) );
      }
      
      if !student.weak_subjects.is_empty()
      {
  println!( "   Areas for Improvement : {}", student.weak_subjects.iter().map( | s | s.name() ).collect::< Vec< _ > >().join( ", " ) );
      }
      
      if !student.learning_goals.is_empty()
      {
  println!( "   Learning Goals:" );
  for ( i, goal ) in student.learning_goals.iter().enumerate()
  {
          println!( "     {}. {}", i + 1, goal );
  }
      }
  }
  else
  {
      println!( "❌ No student currently selected" );
      println!( "💡 Use '/student switch < id >' to select a student" );
  }
  println!();
  }

  /// Register new student (simplified interactive version)
  fn register_new_student( &mut self )
  {
  println!( "\n📝 Student Registration:" );
  println!( "This is a demo - creating sample student 'demo_student'" );
  
  let profile = StudentProfile
  {
      student_id : "demo_student".to_string(),
      name : "Demo Student".to_string(),
      complexity_level : ComplexityLevel::HighSchool,
      learning_style : LearningStyle::Multimodal,
      strong_subjects : vec![ Subject::Mathematics, Subject::Science ],
      weak_subjects : vec![ Subject::History ],
      learning_goals : vec![ "Improve problem-solving skills".to_string() ],
      learning_pace : 1.0,
      preferences : HashMap::new(),
  };
  
  self.platform.register_student( profile );
  println!( "✅ Demo student registered successfully!" );
  println!( "💡 Use '/student switch demo_student' to select this student" );
  }

  /// Conduct interactive assessment
  fn conduct_assessment( &mut self, assessment : &Assessment )
  {
  println!( "\n📝 {} Assessment - {} Level", assessment.subject.name(), assessment.difficulty_level.name() );
  println!( "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" );
  println!( "Total Questions : {} | Total Points : {} | Time Limit : {} minutes", 
             assessment.questions.len(), assessment.total_points, assessment.time_limit_minutes.unwrap_or( 0 ) );
  println!( "Type 'skip' to skip a question, or 'quit' to end assessment early.\n" );
  
  let mut responses = Vec::new();
  
  for ( i, question ) in assessment.questions.iter().enumerate()
  {
      println!( "Question {} ({} points):", i + 1, question.points );
      println!( "{}\n", question.question );
      
      print!( "Your answer : " );
      io::stdout().flush().unwrap_or( () );
      
      let mut answer = String::new();
      io::stdin().read_line( &mut answer ).unwrap_or( 0 );
      let answer = answer.trim().to_string();
      
      if answer.to_lowercase() == "quit"
      {
  println!( "Assessment ended early.\n" );
  break;
      }
      
      responses.push( if answer.to_lowercase() == "skip" { "".to_string() } else { answer } );
  }
  
  let result = self.platform.evaluate_assessment( assessment, responses );
  self.display_assessment_results( &result );
  }

  /// Display assessment results
  fn display_assessment_results( &self, result : &AssessmentResult )
  {
  println!( "\n🎯 Assessment Results:" );
  println!( "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" );
  println!( "Score : {} points ({:.1}%)", result.score, result.percentage );
  println!( "Time Taken : {:.1} minutes", result.time_taken );
  println!( "Performance : {}\n", result.performance_summary );
  
  let correct_count = result.question_feedback.iter().filter( | f | f.is_correct ).count();
  println!( "Question Breakdown : {}/{} correct", correct_count, result.question_feedback.len() );
  
  for ( i, feedback ) in result.question_feedback.iter().enumerate()
  {
      let status = if feedback.is_correct { "✅" } else { "❌" };
      println!( "  {}) {} ({} pts) {}", i + 1, status, feedback.points_earned, feedback.feedback );
  }
  
  if !result.improvement_areas.is_empty()
  {
      println!( "\n📈 Areas for Improvement:" );
      for area in &result.improvement_areas
      {
  println!( "   • {}", area );
      }
  }
  
  if !result.recommendations.is_empty()
  {
      println!( "\n💡 Recommendations:" );
      for rec in &result.recommendations
      {
  println!( "   • {}", rec );
      }
  }
  println!();
  }

  /// Display student progress
  fn display_student_progress( &self, progress : &HashMap< Subject, LearningProgress > )
  {
  println!( "\n📊 Learning Progress:" );
  println!( "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" );
  
  for ( subject, prog ) in progress
  {
      let accuracy = if prog.total_questions > 0 { ( prog.correct_answers as f32 / prog.total_questions as f32 ) * 100.0 } else { 0.0 };
      println!( "📚 {}:", subject.name() );
      println!( "   Proficiency : {:.1}% | Accuracy : {:.1}% ({}/{})", 
               prog.proficiency_level * 100.0, accuracy, prog.correct_answers, prog.total_questions );
      println!( "   Concepts Mastered : {} | Last Updated : {}", prog.concepts_mastered, prog.last_updated );
  }
  println!();
  }

  /// Display platform statistics  
  fn display_platform_statistics( &self )
  {
  let stats = self.platform.get_statistics();
  
  println!( "🎓 Platform Statistics:" );
  println!( "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" );
  println!( "Students Served : {}", stats.students_served );
  println!( "Total Explanations : {}", stats.total_explanations );
  println!( "Questions Answered : {}", stats.total_questions_answered );
  println!( "Total Learning Time : {} minutes", stats.total_learning_time );
  
  if !stats.subject_popularity.is_empty()
  {
      println!( "\nSubject Popularity:" );
      let mut subjects : Vec< _ > = stats.subject_popularity.iter().collect();
      subjects.sort_by( | a, b | b.1.cmp( a.1 ) );
      for ( subject, count ) in subjects.iter().take( 5 )
      {
  println!( "   {}: {} requests", subject.name(), count );
      }
  }
  println!();
  }

  /// Register sample students for demonstration
  fn register_sample_students( &mut self )
  {
  let students = vec![
      StudentProfile
      {
  student_id : "alice_hs".to_string(),
  name : "Alice Johnson".to_string(),
  complexity_level : ComplexityLevel::HighSchool,
  learning_style : LearningStyle::Visual,
  strong_subjects : vec![ Subject::Mathematics, Subject::Science ],
  weak_subjects : vec![ Subject::History, Subject::LanguageArts ],
  learning_goals : vec![ "Master calculus concepts".to_string(), "Improve essay writing".to_string() ],
  learning_pace : 1.2,
  preferences : HashMap::new(),
      },
      StudentProfile
      {
  student_id : "bob_ms".to_string(),
  name : "Bob Smith".to_string(),
  complexity_level : ComplexityLevel::MiddleSchool,
  learning_style : LearningStyle::Kinesthetic,
  strong_subjects : vec![ Subject::Science, Subject::Arts ],
  weak_subjects : vec![ Subject::Mathematics ],
  learning_goals : vec![ "Improve math problem solving".to_string() ],
  learning_pace : 0.8,
  preferences : HashMap::new(),
      },
  ];

  for student in students
  {
      self.platform.register_student( student );
  }

  // Set first student as current
  self.platform.set_current_student( Some( "alice_hs".to_string() ) );
  }

  /// Print welcome message
  fn print_welcome( &self )
  {
  println!( "🎓 Welcome to the Educational AI Tutor!" );
  println!( "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" );
  println!( "An intelligent tutoring system powered by HuggingFace's language models." );
  println!( "Personalized learning assistance across multiple subjects and complexity levels.\n" );
  println!( "Type /help for available commands or start with:" );
  println!( "  • /explain < concept > - Get detailed explanations" );
  println!( "  • /ask < question > - Ask questions and get tutoring responses" );
  println!( "  • /student list - See available students" );
  println!( "  • /assess < subject > - Take subject assessments\n" );
  }

  /// Print help information
  fn print_help( &self ) -> Result< (), Box< dyn std::error::Error > >
  {
  println!( "\n🎓 Educational AI Tutor - Command Reference:" );
  println!( "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" );
  
  println!( "\n📖 Learning Commands:" );
  println!( "  /explain < concept >     - Generate detailed concept explanations" );
  println!( "                          Example : /explain photosynthesis" );
  println!( "  /ask < question >        - Ask questions and receive tutoring responses" );
  println!( "                          Example : /ask What is the quadratic formula?" );
  
  println!( "\n👥 Student Management:" );
  println!( "  /student list          - List all registered students" );
  println!( "  /student switch < id >   - Switch to different student profile" );
  println!( "  /student profile       - Show current student details" );
  println!( "  /student register      - Register new student (demo mode)" );
  
  println!( "\n📝 Assessment & Progress:" );
  println!( "  /assess < subject >      - Generate and take assessments" );
  println!( "                          Subjects : math, science, history, english, cs, art" );
  println!( "  /progress              - View learning progress and statistics" );
  
  println!( "\n⚙️  System Commands:" );
  println!( "  /config                - Show platform configuration" );
  println!( "  /help                  - Show this help message" );
  println!( "  /quit or /exit         - Exit the tutor" );
  
  println!( "\n🎯 Features:" );
  println!( "  • Adaptive complexity levels (Elementary → Graduate)" );
  println!( "  • Multiple learning styles (Visual, Auditory, Kinesthetic, etc.)" );
  println!( "  • Subject specialization (Math, Science, History, etc.)" );
  println!( "  • Progress tracking and personalized recommendations" );
  println!( "  • Interactive assessments with detailed feedback" );
  println!( "  • Real-time response generation using HuggingFace models" );
  
  println!( "\n💡 Tips:" );
  println!( "  • Switch between students to see how responses adapt" );
  println!( "  • Try explaining the same concept at different complexity levels" );
  println!( "  • Take assessments to track learning progress" );
  println!( "  • Ask follow-up questions for deeper understanding" );
  println!();
  
  Ok( () )
  }
}

/// Main function - entry point for the educational AI tutor example
#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  // Load API key from environment or workspace secrets
  let api_key = std::env::var("HUGGINGFACE_API_KEY")
  .or_else(|_| {
      use workspace_tools as workspace;
      let workspace = workspace::workspace()
  .map_err(|_| std::env::VarError::NotPresent)?; // Convert WorkspaceError
      let secrets = workspace.load_secrets_from_file("-secrets.sh")
  .map_err(|_| std::env::VarError::NotPresent)?; // Convert WorkspaceError
      secrets.get("HUGGINGFACE_API_KEY")
  .cloned()
  .ok_or(std::env::VarError::NotPresent)
  })
  .map_err(|_| "HUGGINGFACE_API_KEY not found in environment or workspace secrets. Get your API key from https://huggingface.co/settings/tokens")?;

  // Initialize client
  let secret = Secret::new( api_key );
  let env = HuggingFaceEnvironmentImpl::build( secret, None )?;
  let client = Client::build( env )?;

  // Create tutor platform
  let platform = AITutorPlatform::new( client );

  // Create and run CLI
  let mut cli = EducationalTutorCLI::new( platform );
  cli.run().await?;

  println!( "\n🎓 Thank you for using the Educational AI Tutor!" );
  println!( "Keep learning and exploring new concepts! 📚✨" );

  Ok( () )
}