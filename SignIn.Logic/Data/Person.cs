﻿using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using System.Threading.Tasks;

namespace SignIn.Logic.Data
{
    public class Person : NotifyBase
    {
        #region People

        public static TimeSpan cacheExpiration = new TimeSpan(2, 0, 0);
        static DateTime? lastRefresh = null;

        static List<Person> peopleCache;
        static Task<List<Person>> getPeopleTask = null;

        public static async Task<IEnumerable<Person>> GetPeople(bool alwaysUseCache = true)
        {
            if (!alwaysUseCache && DateTime.Now - lastRefresh > cacheExpiration)
                peopleCache = null;

            if (getPeopleTask != null && !getPeopleTask.IsCompleted)
                await getPeopleTask;

            if (peopleCache == null)
            {
                lastRefresh = DateTime.Now;
                getPeopleTask = Task.Run(async () => peopleCache = await DataProvider.Current.GetPeople());
                peopleCache = await getPeopleTask;
            }

            return peopleCache.OrderBy(p => p.FirstName).ThenBy(p => p.LastName);
        }

        public void Clear()
        {
            if (Guardian == null)
                Guardian = new Guardian();
            else
                Guardian.Clear();

            if (Address == null)
                Address = new Address();
            else
                Address.Clear();
        }

        public static void ClearPeopleCache() => peopleCache = null;

        #endregion

        public Person() { }

        public Person(string id, string firstName, string lastName, string notes, DateTimeOffset? birthDate = null, Address address = null)
            : this()
        {
            Id = id;
            FirstName = firstName;
            LastName = lastName;
            Notes = notes;

            BirthDate = birthDate ?? DateTime.Now;
            Grade = "";
            Address = address;

            UpdateFromNotes();
        }

        string id;
        public string Id
        {
            get => id;
            set { id = value; OnPropertyChanged(); }
        }

        string firstName;
        public string FirstName
        {
            get => firstName;
            set { firstName = value; OnPropertyChanged(); }
        }

        string lastName;
        public string LastName
        {
            get => lastName;
            set { lastName = value; OnPropertyChanged(); }
        }

        DateTimeOffset birthDate = DateTimeOffset.Now;
        public DateTimeOffset BirthDate
        {
            get => birthDate;
            set { birthDate = value; OnPropertyChanged(); }
        }

        const string GradeLabel = "Grade: ";
        readonly static Regex GradeRegex = new Regex(NoteParser.GetLabelExpression(GradeLabel), RegexOptions.Compiled);
        public static readonly string[] grades = new[] { "Kindergarten", "1st", "2nd", "3rd", "4th", "5th", "6th", "7th", "8th", "9th", "10th", "11th", "12th", "Finished" };
        public string[] Grades => grades;

        string grade;
        public string Grade
        {
            get => grade;
            set
            {
                if (value == grade)
                    return;

                grade = value;
                OnPropertyChanged();
            }
        }

        Address address = new Address();
        public Address Address
        {
            get => address;
            set { address = value; OnPropertyChanged(); }
        }

        Guardian guardian;
        public Guardian Guardian
        {
            get => guardian;
            set { guardian = value; OnPropertyChanged(); }
        }

        string notes;
        public string Notes
        {
            get => notes;
            set { notes = value; OnPropertyChanged(); }
        }

        public string FullName => $"{FirstName} {LastName}";

        public async Task<bool> Save(bool isUpdating = false)
        {
            try
            {
                if (!isUpdating && !IsValid(out string personIssues))
                    throw new InvalidOperationException(personIssues);

                string guardianIssues = null;
                if (Guardian != null)
                {
                    if (!Guardian.IsValid(out guardianIssues))
                        throw new InvalidOperationException(guardianIssues ?? "There must be a guardian to save!");

                    Guardian.LastUpdated = DateTime.Today;
                }

                UpdateToNotes();

                var newId = await DataProvider.Current.SavePerson(this);
                Id = newId;

                ClearPeopleCache();
                return true;
            }
            catch (Exception ex)
            {
                await DataProvider.Current.ShowMessage("Oops! Something went wrong while signing you up. Sorry about that...", ex);
                return false;
            }
        }

        public bool IsValid(out string issues)
        {
            issues = "";

            var emptyFields = new List<string>();
            if (string.IsNullOrWhiteSpace(FirstName))
                emptyFields.Add("first name");
            if (string.IsNullOrWhiteSpace(LastName))
                emptyFields.Add("last name");
            if (string.IsNullOrWhiteSpace(Grade))
                emptyFields.Add("grade");
            if (Address?.IsValid() != true)
                emptyFields.Add("address");

            if (emptyFields.Any())
            {
                issues = "Please enter your ";
                if (emptyFields.Count == 1)
                    issues += emptyFields[0];
                else
                {
                    issues += string.Join(", ", emptyFields.Take(emptyFields.Count - 1));
                    issues += $" and {emptyFields.Last()}";
                }

                issues += ".\r\n";
            }

            if (BirthDate.CompareTo(DateTimeOffset.Now.AddYears(-5)) > 0)
                issues += "You have to be at least five to sign up.";

            return string.IsNullOrWhiteSpace(issues);
        }

        public bool IsInfoExpired => Guardian.IsInfoExpired;

        public bool SkipNextExpire { get; set; }

        void UpdateFromNotes()
        {
            Grade = NoteParser.FromNotes(Notes, GradeRegex).FirstOrDefault() ?? "";
            Guardian = Guardian.FromNotes(Notes);
        }

        void UpdateToNotes()
        {
            var grade = NoteParser.ToNotes((GradeLabel, Grade));
            var guardianNotes = Guardian?.ToString();
            Notes = NoteParser.JoinValues(new[] { grade, guardianNotes });
        }

        #region Signed in

        public async Task RefreshSignedIn()
        {
            var todaysLogs = await Log.GetLogs(DateTime.Today);
            SignedInLog = todaysLogs.FirstOrDefault(l => l.PersonId == Id && l.SignedIn);
        }

        Log signedInLog;
        public Log SignedInLog
        {
            get => signedInLog;
            set { signedInLog = value; OnPropertyChanged(); OnPropertyChanged(nameof(SignedIn)); }
        }

        public bool SignedIn => SignedInLog != null;

        public async Task<SignInOutResult> SignInOut()
        {
            try
            {
                if (IsInfoExpired)
                {
                    if (SkipNextExpire)
                        SkipNextExpire = false;
                    else
                        return SignInOutResult.InfoExpired;
                }

                await RefreshSignedIn();

                if (SignedInLog == null)
                    SignedInLog = await Log.New(this);
                else
                    await SignedInLog.SignOut();

                await RefreshSignedIn();

                return SignInOutResult.Success;
            }
            catch (Exception ex)
            {
                await DataProvider.Current.ShowMessage($"Oops, something went wrong. Try again later.\r\n{ex.GetBaseException().Message}");
                return SignInOutResult.Failed;
            }
        }

        #endregion

        public override string ToString() => FullName;
    }

    public enum SignInOutResult
    {
        Success,
        Failed,
        InfoExpired
    }
}