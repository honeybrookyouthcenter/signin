﻿using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace YouthCenterSignIn.Logic.Data
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

        public static void ClearPeopleCache() => peopleCache = null;

        #endregion

        public Person() { }

        public Person(string id, string firstName, string lastName, DateTimeOffset? birthDate = null, Address address = null, Guardian guardian = null) : this()
        {
            Id = id;
            FirstName = firstName;
            LastName = lastName;
            BirthDate = birthDate ?? DateTime.Now;
            Address = address;
            Guardian = guardian;
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

        public string FullName => $"{FirstName} {LastName}";

        public async Task<bool> Save()
        {
            try
            {
                IsSaving = true;

                if (!IsValid(out string personIssues))
                    throw new InvalidOperationException(personIssues);

                string guardianIssues = null;
                if (Guardian?.IsValid(out guardianIssues) != true)
                    throw new InvalidOperationException(guardianIssues ?? "There must be a guardian to save!");

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
            finally
            {
                IsSaving = false;
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

        bool isSaving;
        public bool IsSaving
        {
            get => isSaving;
            private set { isSaving = value; OnPropertyChanged(); }
        }

        public async Task SignInOut()
        {
            await RefreshSignedIn();

            if (SignedInLog == null)
                SignedInLog = await Log.New(this);
            else
                await SignedInLog.SignOut();

            await RefreshSignedIn();
        }

        #endregion
    }
}
