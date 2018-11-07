﻿using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Threading.Tasks;

namespace YouthCenterSignIn.Logic.Data
{
    public class Person : INotifyPropertyChanged
    {
        #region People

        static List<Person> peopleCache;
        static Task<List<Person>> getPeopleTask = null;

        public static async Task<IEnumerable<Person>> GetPeople()
        {
            if (getPeopleTask != null && !getPeopleTask.IsCompleted)
                await getPeopleTask;

            if (peopleCache == null)
            {
                getPeopleTask = Task.Run(async () => peopleCache = await DataProvider.Current.GetPeople());
                peopleCache = await getPeopleTask;
            }

            return peopleCache.OrderBy(p => p.FirstName).ThenBy(p => p.LastName);
        }

        #endregion

        public Person() { }

        public Person(string id, string firstName, string lastName)
        {
            Id = id;
            FirstName = firstName;
            LastName = lastName;
        }

        public string Id { get; }

        public string FirstName { get; set; }

        public string LastName { get; set; }

        public DateTimeOffset BirthDate { get; set; }

        public string PhoneNumber { get; set; }

        public string Address { get; set; }

        public string Notes { get; set; }

        public string FullName => $"{FirstName} {LastName}";

        public override string ToString() => $"{Id}: {FullName}";

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

        #region Notify

        /// <summary>
        /// Property Changed event
        /// </summary>
        public event PropertyChangedEventHandler PropertyChanged;

        /// <summary>
        /// Fire the PropertyChanged event
        /// </summary>
        /// <param name="propertyName">Name of the property that changed (defaults from CallerMemberName)</param>
        protected void OnPropertyChanged([System.Runtime.CompilerServices.CallerMemberName] string propertyName = "")
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }

        #endregion
    }
}